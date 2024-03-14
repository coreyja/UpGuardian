use std::time::Duration;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use chrono::{DateTime, Utc};
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::{html, Render};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, templates::IntoTemplate};

use super::sites::Site;

pub async fn new(
    site: Site,
    session: DBSession,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    html! {
      h1 { "New Page" }

      form method="post" action=(format!("/my/sites/{}/pages", site.site_id)) {
        label {
          "Path"
          input type="text" name="path" required;
        }

        label {
          "Name"
          input type="text" name="name" required;
        }

        button type="submit" { "Create" }
      }
    }
    .into_template(app_state, Some(session))
    .await
    .unwrap()
}

#[derive(serde::Deserialize)]
pub struct PageFormData {
    path: String,
    name: String,
}

pub async fn create(
    site: Site,
    State(state): State<AppState>,
    Form(form_data): Form<PageFormData>,
) -> impl IntoResponse {
    let site_id = site.site_id;

    sqlx::query!(
        r#"
    INSERT INTO Pages (site_id, path, name)
    VALUES ($1, $2, $3)
  "#,
        site_id,
        form_data.path,
        form_data.name
    )
    .execute(state.db())
    .await
    .unwrap();

    Redirect::to(&format!("/my/sites/{}", site_id,))
}

pub struct Page {
    pub page_id: Uuid,
    pub site_id: Uuid,
    pub path: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct PagePath {
    page_id: Uuid,
}

pub async fn show(
    site: Site,
    State(state): State<AppState>,
    Path(PagePath { page_id }): Path<PagePath>,
    session: DBSession,
) -> impl IntoResponse {
    let page = sqlx::query_as!(
        Page,
        r#"
    SELECT Pages.*
    FROM Pages
    JOIN Sites ON Sites.site_id = Pages.site_id
    WHERE Pages.page_id = $1 AND Pages.site_id = $2 AND Sites.user_id = $3
  "#,
        page_id,
        site.site_id,
        session.user_id
    )
    .fetch_one(state.db())
    .await
    .unwrap();

    let checkins = sqlx::query_as!(
        Checkin,
        r#"
      SELECT *
      FROM Checkins
      WHERE page_id = $1
      ORDER BY created_at DESC
      LIMIT 10
    "#,
        page_id
    )
    .fetch_all(state.db())
    .await
    .unwrap();

    let mut checkins_for_graph = checkins.clone();
    checkins_for_graph.reverse();
    let graph = CheckinGraph {
        checkins: checkins_for_graph,
    };

    html! {
      h1 { (page.name) }

      p { (page.path) }

      h2 { "Checkins" }

      (graph)

      ul {
        @for checkin in checkins {
          li {
            (checkin.created_at.format("%d/%m/%Y %H:%M:%S")) " - " (checkin.outcome)
            @if let Some(status) = checkin.status_code {
              " - " (status)
            }
            @if let Some(duration) = checkin.duration_nanos {
              @let duration = Duration::from_nanos(duration as u64);
              " - " (humantime::format_duration(duration))
            }
          }
        }
      }
    }
    .into_template(state, Some(session))
    .await
    .unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Checkin {
    checkin_id: Uuid,
    page_id: Uuid,
    outcome: String,
    status_code: Option<i32>,
    duration_nanos: Option<i64>,
    created_at: DateTime<Utc>,
}

struct CheckinGraph {
    checkins: Vec<Checkin>,
}

struct GraphPoint {
    x: f64,
    y: f64,
    label: String,
}

struct YAxisLine {
    width: usize,
    y_pos: usize,
    label: String,
}

impl Render for YAxisLine {
    fn render(&self) -> maud::Markup {
        let Self {
            width,
            y_pos,
            label,
        } = self;

        html! {
          path d=(format!("M0 {0} L{width} {0}", y_pos))  fill="none" stroke="blue" stroke-dasharray="2" stroke-width="0.25" {}
          text x=(0) y=(y_pos) font-size="5" fill="blue" { (label) }
        }
    }
}

impl Render for CheckinGraph {
    fn render(&self) -> maud::Markup {
        let full_height = 100;
        let height_padding = 10;

        let width = 200;
        let height = full_height - height_padding * 2;

        let total_points = self.checkins.len();
        let per_point_x = width / total_points;

        let min_duration_nanos = self
            .checkins
            .iter()
            .map(|p| p.duration_nanos.unwrap())
            .min()
            .unwrap()
            / 1_000_000
            * 1_000_000;
        let min_label =
            humantime::format_duration(Duration::from_nanos(min_duration_nanos as u64)).to_string();

        let max_duration_nanos = self
            .checkins
            .iter()
            .map(|p| p.duration_nanos.unwrap())
            .max()
            .unwrap()
            / 1_000_000
            * 1_000_000;
        let max_label =
            humantime::format_duration(Duration::from_nanos(max_duration_nanos as u64)).to_string();

        let height_range = max_duration_nanos - min_duration_nanos;

        let points = self
            .checkins
            .iter()
            .enumerate()
            .map(|(i, p)| GraphPoint {
                x: (per_point_x * i) as f64,
                y: ((full_height - height_padding) as f64)
                    - (((p.duration_nanos.unwrap() as f64 - min_duration_nanos as f64)
                        / height_range as f64)
                        * height as f64),
                label: humantime::format_duration(Duration::from_nanos(
                    p.duration_nanos.unwrap() as u64
                ))
                .to_string(),
            })
            .collect::<Vec<_>>();

        let svg_path = points
            .iter()
            .enumerate()
            .map(|(i, GraphPoint { x, y, .. })| {
                if i == 0 {
                    format!("M{x} {y}")
                } else {
                    format!("L{x} {y}")
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        html! {
          svg class="w-full" viewBox="0 0 200 100" {

            // y max line
            (YAxisLine { width, y_pos: height_padding, label: format!("Max: {max_label}")})

            // y max line
            (YAxisLine { width, y_pos: full_height - height_padding, label: format!("Min: {min_label}")})

            // This is the actual point line
            path d=(svg_path) fill="none" stroke="black" {}

            @for GraphPoint { x, y, label } in points.iter() {
              // Group for Hover State
              g class="group"  {
                // Invisible Circle to make hover state bigger
                circle cx=(x) cy=(y) r=(4) class="fill-transparent stroke-transparent" {}
                // Point on line
                circle cx=(x) cy=(y) r=(2) class="group-hover:fill-red-500" {}

                // Label, hidden till group hover
                text x=(x) y=(y + 5.0) font-size=4 class="hidden group-hover:block fill-red-500" { (label) }
              }
            }
          }
        }
    }
}
