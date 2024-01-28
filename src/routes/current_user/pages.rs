use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::html;
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

    let checkins = sqlx::query!(
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

    html! {
      h1 { (page.name) }

      p { (page.path) }

      h2 { "Checkins" }

      ul {
        @for checkin in checkins {
          li {
            (checkin.created_at) " - " (checkin.outcome)
            @if let Some(status) = checkin.status_code {
              " - " (status)
            }
          }
        }
      }
    }
    .into_template(state, Some(session))
    .await
    .unwrap()
}
