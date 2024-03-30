use std::{fmt::Display, time::Duration};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Form,
};
use chrono::{DateTime, Utc};
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::{html, Render};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use uuid::Uuid;

use crate::{
    app_state::AppState, routes::current_user::sites::single_stat, templates::IntoTemplate,
};

use super::sites::{
    avg_response_time_for_checkins, calculate_percentile_change, calculate_response_time_change,
    chrono_to_pg_interval, success_percent_for_checkins, Site,
};

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

    let all_checkins = sqlx::query_as!(
        Checkin,
        r#"
      SELECT *
      FROM Checkins
      WHERE page_id = $1
      AND created_at >= now() - INTERVAL '12 hours'
      AND duration_nanos is not null
      ORDER BY created_at DESC
    "#,
        page_id
    )
    .fetch_all(state.db())
    .await
    .unwrap();
    let recent_duration = chrono::Duration::hours(6);
    let split_point = all_checkins
        .iter()
        .position(|checkin| checkin.created_at < chrono::Utc::now() - recent_duration)
        .unwrap_or(all_checkins.len());

    let mut new_checkins = all_checkins;
    let old_checkins = new_checkins.split_off(split_point);
    let new_checkins = new_checkins;

    let avg_response_time = avg_response_time_for_checkins(&new_checkins);

    let response_time_change = calculate_response_time_change(&old_checkins, avg_response_time);

    let avg_response_time = format!("{:.1} ms", avg_response_time);

    let succesful_percent = success_percent_for_checkins(&new_checkins);
    let successful_percent_change =
        calculate_percentile_change(old_checkins, &new_checkins, succesful_percent);
    let succesful_percent = format!("{:.1}%", succesful_percent);

    let mut checkins_for_graph = new_checkins.clone();
    checkins_for_graph.reverse();
    let graph = SampledCheckinGraph {
        checkins: checkins_for_graph,
        number_of_chunks: 20,
        range: Some(chrono::Utc::now() - recent_duration..chrono::Utc::now()),
    };

    html! {
      h1 { (page.name) }

      p { (page.path) }

      h2 { "Checkins" }

      form action=(format!("/my/sites/{}/pages/{}/refresh", site.site_id, page.page_id)) method="get" data-target=".refresh" data-app="LiveForm" {
        select name="hours" {
          option value="6" { "6 hours" }
          option value="12" { "12 hours" }
          option value="24" { "24 hours" }
          option value="48" { "48 hours" }
          option value="168" { "1 week" }
        }

        div class="refresh" {

            dl."mt-5 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3" {
                (
                single_stat("# of Checkins", new_checkins.len(), None, "fa-file")
                )
                (
                single_stat("Avg. Response Time", avg_response_time, response_time_change, "fa-clock")
                )
                (
                single_stat("Success Rate", succesful_percent, successful_percent_change, "fa-check")
                )
            }

            div class="graph" {
            (graph)
            }
        }

      }
    }
    .into_template(state, Some(session))
    .await
    .unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphQuery {
    hours: i32,
}

pub trait FromHours {
    fn from_hours(hours: i32) -> Self;
}

impl FromHours for PgInterval {
    fn from_hours(hours: i32) -> Self {
        PgInterval {
            months: 0,
            days: hours / 24,
            microseconds: (hours % 24) as i64 * 60 * 60 * 1_000_000,
        }
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    Path(PagePath { page_id }): Path<PagePath>,
    _session: DBSession,
    Query(GraphQuery { hours }): Query<GraphQuery>,
) -> impl IntoResponse {
    let recent_duration = chrono::Duration::hours(hours.into());
    let interval = chrono_to_pg_interval(recent_duration * 2);

    let all_checkins = sqlx::query_as!(
        Checkin,
        r#"
  SELECT *
  FROM Checkins
  WHERE page_id = $1
  AND now() - created_at <= $2
  AND duration_nanos is not null
  ORDER BY created_at DESC
"#,
        page_id,
        interval
    )
    .fetch_all(state.db())
    .await
    .unwrap();
    let split_point = all_checkins
        .iter()
        .position(|checkin| checkin.created_at < chrono::Utc::now() - recent_duration)
        .unwrap_or(all_checkins.len());

    let mut new_checkins = all_checkins;
    let old_checkins = new_checkins.split_off(split_point);
    let new_checkins = new_checkins;

    let avg_response_time = avg_response_time_for_checkins(&new_checkins);

    let response_time_change = calculate_response_time_change(&old_checkins, avg_response_time);

    let avg_response_time = format!("{:.1} ms", avg_response_time);

    let succesful_percent = success_percent_for_checkins(&new_checkins);
    let successful_percent_change =
        calculate_percentile_change(old_checkins, &new_checkins, succesful_percent);
    let succesful_percent = format!("{:.1}%", succesful_percent);

    let mut checkins_for_graph = new_checkins.clone();

    checkins_for_graph.reverse();
    let graph = SampledCheckinGraph {
        checkins: checkins_for_graph,
        number_of_chunks: 20,
        range: Some(chrono::Utc::now() - recent_duration..chrono::Utc::now()),
    };

    html! {
       dl."mt-5 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3" {
           (
           single_stat("# of Checkins", new_checkins.len(), None, "fa-file")
           )
           (
           single_stat("Avg. Response Time", avg_response_time, response_time_change, "fa-clock")
           )
           (
           single_stat("Success Rate", succesful_percent, successful_percent_change, "fa-check")
           )
       }

       div class="graph" {
        (graph)
       }
    }
    .0
}

struct CheckinTable(Vec<Checkin>);

impl Render for CheckinTable {
    fn render(&self) -> maud::Markup {
        html! {
          ul {
            @for checkin in self.0.iter() {
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
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkin {
    pub checkin_id: Uuid,
    pub page_id: Uuid,
    pub outcome: String,
    pub status_code: Option<i32>,
    pub duration_nanos: Option<i64>,
    pub created_at: DateTime<Utc>,
}

struct SimpleCheckinGraph {
    checkins: Vec<Checkin>,
    range: Option<std::ops::Range<DateTime<Utc>>>,
}

struct SampledCheckinGraph {
    checkins: Vec<Checkin>,
    number_of_chunks: usize,
    range: Option<std::ops::Range<DateTime<Utc>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

struct XAxisTicks {
    width: usize,
    x_range: std::ops::Range<DateTime<Utc>>,
    number_of_ticks: usize,
}

struct SvgPath {
    points: Vec<(f64, f64)>,
    stroke_width: f64,
    path_class: String,
    stroke_dashed: bool,
}

impl Render for SvgPath {
    fn render(&self) -> maud::Markup {
        let Self {
            points,
            stroke_width,
            path_class,
            stroke_dashed,
        } = self;

        let svg_path = points
            .iter()
            .enumerate()
            .map(|(i, (x, y))| {
                if i == 0 {
                    format!("M{x} {y}")
                } else {
                    format!("L{x} {y}")
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        html! {
          path
            d=(svg_path)
            fill="none"
            class=(path_class)
            stroke-width=(stroke_width)
            stroke-dasharray=(if *stroke_dashed { "2" } else { "0" })
            {}
        }
    }
}

struct SvgPathWithPoints {
    points: Vec<GraphPoint>,
    stroke_width: f64,
    path_class: String,
    stroke_dashed: bool,
    point_radius: usize,
    label_font_size: usize,
    group_class: String,
}

impl Render for SvgPathWithPoints {
    fn render(&self) -> maud::Markup {
        let Self {
            point_radius,
            label_font_size,
            group_class,
            points,
            stroke_width,
            path_class,
            stroke_dashed,
        } = self;

        let length = points.len();

        let x_min = points
            .iter()
            .map(|p| p.x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let x_max = points
            .iter()
            .map(|p| p.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let x_midpoint = (x_min + x_max) / 2.0;
        let split_index = points
            .iter()
            .position(|p| p.x > x_midpoint)
            .unwrap_or(length / 2);

        let mut left_side = points.clone();
        let right_side = left_side.split_off(split_index);

        assert_eq!(points.len(), left_side.len() + right_side.len());

        html! {
          (SvgPath {
            points: points.iter().map(|p| (p.x, p.y)).collect(),
            stroke_width: *stroke_width,
            path_class: path_class.clone(),
            stroke_dashed: *stroke_dashed,
          })

          @for GraphPoint { x, y, label } in left_side.iter().rev() {
            g class=(format!("group {group_class}")) {
              circle cx=(x) cy=(y) r=(point_radius) class="fill-transparent stroke-transparent" {}
              circle cx=(x) cy=(y) r=(point_radius / 2) {}

              text
                x=(x)
                y=(y + 5.0)
                font-size=(label_font_size)
                stroke-width="1em"
                stroke-linejoin="round"
                paint-order="stroke"
                text-anchor="start"
                class="hidden group-hover:block stroke-white"
                { (label) }
            }
          }

          @for GraphPoint { x, y, label } in right_side.iter() {
            g class=(format!("group {group_class}")) {
              circle cx=(x) cy=(y) r=(point_radius) class="fill-transparent stroke-transparent" {}
              circle cx=(x) cy=(y) r=(point_radius / 2) {}

              text
                x=(x)
                y=(y + 5.0)
                font-size=(label_font_size)
                stroke-width="1em"
                stroke-linejoin="round"
                paint-order="stroke"
                text-anchor="end"
                class="hidden group-hover:block stroke-white"
                { (label) }
            }
          }
        }
    }
}

impl Render for YAxisLine {
    fn render(&self) -> maud::Markup {
        let Self {
            width,
            y_pos,
            label,
        } = self;

        html! {
          (SvgPath {
            points: vec![
              (0.0,*y_pos as f64),
              (*width as f64, *y_pos as f64),
            ],
            stroke_width: 0.25,
            stroke_dashed: true,
            path_class: "stroke-blue-500".to_string(),
          })
          text x=(0) y=(y_pos) font-size="5" fill="blue" { (label) }
        }
    }
}

impl Render for XAxisTicks {
    fn render(&self) -> maud::Markup {
        let Self {
            width,
            x_range,
            number_of_ticks,
        } = self;

        let tick_width = *width as f64 / *number_of_ticks as f64;

        let ticks = (0..=*number_of_ticks)
            .map(|i| {
                let x = i as f64 * tick_width;
                let time = x_range.start.timestamp() as f64
                    + (x / *width as f64) * (x_range.end - x_range.start).num_seconds() as f64;
                let time = chrono::DateTime::<Utc>::from_timestamp(time as i64, 0).unwrap();
                let label = time.format("%D %H:%M");

                html! {
                  g {
                    (SvgPath {
                      points: vec![
                        (x, 90.0),
                        (x, 95.0),
                      ],
                      stroke_width: 0.25,
                      stroke_dashed: false,
                      path_class: "stroke-blue-500".to_string(),
                    })
                    text x=(x) y=(100) font-size="3" fill="blue" { (label) }
                  }
                }
            })
            .collect::<Vec<_>>();

        maud::html! {
            @for tick in ticks {
                (tick)
            }
        }
    }
}

trait ToFloat {
    fn to_f64(&self) -> f64;
}

impl ToFloat for i64 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

impl ToFloat for Duration {
    fn to_f64(&self) -> f64 {
        self.as_nanos() as f64
    }
}

impl ToFloat for chrono::Duration {
    fn to_f64(&self) -> f64 {
        self.to_std().unwrap().to_f64()
    }
}

fn calculate_range_percentile<T, SubOut>(range: &std::ops::Range<T>, item: T) -> f64
where
    T: PartialOrd + std::ops::Sub<Output = SubOut> + Copy + Display,
    SubOut: ToFloat,
{
    calculate_percentile(range.start, range.end, item)
}

fn calculate_percentile<T, SubOut>(range_start: T, range_end: T, item: T) -> f64
where
    T: PartialOrd + std::ops::Sub<Output = SubOut> + Copy + Display,
    SubOut: ToFloat,
{
    if item < range_start || item > range_end {
        panic!(
            "Item is not within the range. {} {} {}",
            item, range_start, range_end
        );
    }

    let range_size = range_end - range_start; // +1 to include both ends
    let position = item - range_start; // Position of item in the range

    // Calculate percentile
    (position.to_f64()) / (range_size.to_f64())
}

impl Render for SimpleCheckinGraph {
    fn render(&self) -> maud::Markup {
        let full_height = 100;
        let height_padding = 10;

        let width = 200;
        let height = full_height - height_padding * 2;

        let (min_time, max_time) = self
            .range
            .as_ref()
            .map(|r| (r.start, r.end))
            .unwrap_or_else(|| {
                (
                    self.checkins.iter().map(|p| p.created_at).min().unwrap(),
                    self.checkins.iter().map(|p| p.created_at).max().unwrap(),
                )
            });

        let x_range = min_time..max_time;

        let min_duration_nanos = self
            .checkins
            .iter()
            .map(|p| p.duration_nanos.unwrap())
            .min()
            .unwrap()
            / 1_000_000
            * 1_000_000
            - 1_000_000;
        let min_label =
            humantime::format_duration(Duration::from_nanos(min_duration_nanos as u64)).to_string();

        let max_duration_nanos = self
            .checkins
            .iter()
            .map(|p| p.duration_nanos.unwrap())
            .max()
            .unwrap()
            / 1_000_000
            * 1_000_000
            + 1_000_000;

        let max_label =
            humantime::format_duration(Duration::from_nanos(max_duration_nanos as u64)).to_string();

        let y_range = min_duration_nanos..max_duration_nanos;

        let calculate_x =
            |time: DateTime<Utc>| calculate_range_percentile(&x_range, time) * width as f64;

        let calculate_y = |duration: i64| {
            height as f64 + height_padding as f64
                - calculate_range_percentile(&y_range, duration) * height as f64
        };

        let points = self
            .checkins
            .iter()
            .map(|p| GraphPoint {
                x: calculate_x(p.created_at),
                y: calculate_y(p.duration_nanos.unwrap()),
                label: humantime::format_duration(Duration::from_nanos(
                    p.duration_nanos.unwrap() as u64
                ))
                .to_string(),
            })
            .collect::<Vec<_>>();

        html! {
          svg class="w-full" viewBox="0 0 200 100" {

            (XAxisTicks { width, x_range, number_of_ticks: 5 })

            (YAxisLine { width, y_pos: height_padding, label: format!("Max: {max_label}")})

            (YAxisLine { width, y_pos: full_height - height_padding, label: format!("Min: {min_label}")})

            (SvgPathWithPoints {
              points,
              stroke_width: 0.5,
              path_class: "stroke-black".to_string(),
              stroke_dashed: false,
              point_radius: 2,
              label_font_size: 4,
              group_class: "hover:fill-red-500".to_string(),
            })
          }
        }
    }
}

fn transpose<X, Y, Z>(vec: Vec<(X, Y, Z)>) -> (Vec<X>, Vec<Y>, Vec<Z>)
where
    X: Clone,
    Y: Clone,
    Z: Clone,
{
    vec.into_iter().fold(
        (Vec::new(), Vec::new(), Vec::new()),
        |(mut v1, mut v2, mut v3), (e1, e2, e3)| {
            v1.push(e1);
            v2.push(e2);
            v3.push(e3);
            (v1, v2, v3)
        },
    )
}

impl Render for SampledCheckinGraph {
    fn render(&self) -> maud::Markup {
        if self.checkins.is_empty() {
            return html! { "No data found" };
        }
        let full_height = 100;
        let height_padding = 10;

        let width = 200;
        let height = full_height - height_padding * 2;

        let (min_time, max_time) = self
            .range
            .as_ref()
            .map(|r| (r.start, r.end))
            .unwrap_or_else(|| {
                (
                    self.checkins.iter().map(|p| p.created_at).min().unwrap(),
                    self.checkins.iter().map(|p| p.created_at).max().unwrap(),
                )
            });

        let x_range = min_time..max_time;

        let min_duration_nanos = self
            .checkins
            .iter()
            .map(|p| p.duration_nanos.unwrap())
            .min()
            .unwrap()
            / 1_000_000
            * 1_000_000
            - 1_000_000;
        let min_label =
            humantime::format_duration(Duration::from_nanos(min_duration_nanos as u64)).to_string();

        let max_duration_nanos = self
            .checkins
            .iter()
            .map(|p| p.duration_nanos.unwrap())
            .max()
            .unwrap()
            / 1_000_000
            * 1_000_000
            + 1_000_000;

        let max_label =
            humantime::format_duration(Duration::from_nanos(max_duration_nanos as u64)).to_string();

        let y_range = min_duration_nanos..max_duration_nanos;

        let calculate_x =
            |time: DateTime<Utc>| calculate_range_percentile(&x_range, time) * width as f64;

        let calculate_y = |duration: i64| {
            height as f64 + height_padding as f64
                - calculate_range_percentile(&y_range, duration) * height as f64
        };

        let total_count = self.checkins.len();

        let chunk_size = (total_count / self.number_of_chunks).max(1);
        let chunks = self.checkins.chunks(chunk_size);

        let min_avg_max: Vec<((f64, f64), GraphPoint, (f64, f64))> = chunks
            .map(|chunk| {
                let min = chunk
                    .iter()
                    .map(|p| p.duration_nanos.unwrap())
                    .min()
                    .unwrap();
                let max = chunk
                    .iter()
                    .map(|p| p.duration_nanos.unwrap())
                    .max()
                    .unwrap();
                let avg = chunk.iter().map(|p| p.duration_nanos.unwrap()).sum::<i64>()
                    / chunk.len() as i64;

                let x = calculate_x(chunk[chunk.len() / 2].created_at);

                (
                    (x, calculate_y(min)),
                    GraphPoint {
                        x,
                        y: calculate_y(avg),
                        label: humantime::format_duration(Duration::from_nanos(avg as u64))
                            .to_string(),
                    },
                    (x, calculate_y(max)),
                )
            })
            .collect();

        let (min, avg, max) = transpose(min_avg_max);

        html! {
          svg class="w-full" viewBox="0 0 200 100" {

            (XAxisTicks { width, x_range, number_of_ticks: 5 })

            (YAxisLine { width, y_pos: height_padding, label: format!("Max: {max_label}")})

            (YAxisLine { width, y_pos: full_height - height_padding, label: format!("Min: {min_label}")})

            (SvgPath {
              points: min,
              stroke_width: 0.25,
              path_class: "stroke-black".to_string(),
              stroke_dashed: true,
            })

            (SvgPath {
              points: max,
              stroke_width: 0.25,
              path_class: "stroke-black".to_string(),
              stroke_dashed: true,
            })

            (SvgPathWithPoints {
              points: avg,
              stroke_width: 0.5,
              path_class: "stroke-blue-500".to_string(),
              stroke_dashed: false,
              point_radius: 2,
              label_font_size: 4,
              group_class: "hover:fill-red-500".to_string(),
            })
          }
        }
    }
}
