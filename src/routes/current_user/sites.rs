use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_macros::debug_handler;
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::html;

use crate::app_state::AppState;

pub async fn index(_session: DBSession) -> impl IntoResponse {
    html! {
      h1 { "My Sites" }

      a href="/my/sites/new" { "Create a new site" }
    }
}

pub async fn new(_session: DBSession) -> impl IntoResponse {
    html! {
      h1 { "New Site" }

      form method="post" action="/my/sites" {
        label {
          "Name"
          input type="text" name="name" required;
        }

        label {
          "Domain"
          input type="text" name="domain" required;
        }

        label {
          "Description"
          textarea name="description" {}
        }

        input type="submit" value="Create";
      }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateSiteFormData {
    name: String,
    domain: String,
    description: Option<String>,
}

pub async fn create(
    session: DBSession,
    State(app_state): State<AppState>,
    Form(form_data): Form<CreateSiteFormData>,
) -> impl IntoResponse {
    let new_site_id = sqlx::query!(
        r#"
      INSERT INTO Sites (name, domain, description, user_id)
      VALUES ($1, $2, $3, $4)
      RETURNING id
    "#,
        form_data.name,
        form_data.domain,
        form_data.description,
        session.user_id
    )
    .fetch_one(app_state.db())
    .await
    .unwrap()
    .id;

    Redirect::to(&format!("/my/sites/{new_site_id}"))
}
