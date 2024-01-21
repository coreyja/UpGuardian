use axum::{
    extract::{FromRequestParts, Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::html;
use uuid::Uuid;

use crate::app_state::AppState;

use super::sites::Site;

pub async fn new(site: Site) -> impl IntoResponse {
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
) -> impl IntoResponse {
    let page = sqlx::query_as!(
        Page,
        r#"
    SELECT *
    FROM Pages
    WHERE page_id = $1 AND site_id = $2
  "#,
        page_id,
        site.site_id
    )
    .fetch_one(state.db())
    .await
    .unwrap();

    html! {
      h1 { (page.name) }

      p { (page.path) }
    }
}
