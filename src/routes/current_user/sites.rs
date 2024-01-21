use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_macros::debug_handler;
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::html;
use serde::Deserialize as _;
use uuid::Uuid;

use crate::app_state::AppState;

pub async fn index(session: DBSession, State(state): State<AppState>) -> impl IntoResponse {
    let sites = sqlx::query!(
        r#"
    SELECT site_id, name, domain, description
    FROM Sites
    WHERE user_id = $1
  "#,
        session.user_id
    )
    .fetch_all(state.db())
    .await
    .unwrap();

    html! {
      h1 { "My Sites" }

      a href="/my/sites/new" { "Create a new site" }

      ul {
        @for site in sites {
          li {
            a href=(format!("/my/sites/{}", site.site_id)) title=[site.description] {
              (site.name) " - " (site.domain)
            }
          }
        }
      }
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
    #[serde(deserialize_with = "empty_string_is_none")]
    description: Option<String>,
}

fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
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
      RETURNING site_id
    "#,
        form_data.name,
        form_data.domain,
        form_data.description,
        session.user_id
    )
    .fetch_one(app_state.db())
    .await
    .unwrap()
    .site_id;

    Redirect::to(&format!("/my/sites/{new_site_id}"))
}

pub async fn show(
    session: DBSession,
    State(app_state): State<AppState>,
    Path(site_id): Path<Uuid>,
) -> axum::response::Response {
    let site = sqlx::query!(
        r#"
      SELECT site_id, name, domain, description
      FROM Sites
      WHERE site_id = $1 AND user_id = $2
    "#,
        site_id,
        session.user_id
    )
    .fetch_optional(app_state.db())
    .await
    .unwrap();

    let Some(site) = site else {
        return (StatusCode::NOT_FOUND, "Site not found").into_response();
    };

    html! {
      h1 { (site.name) }

      @if let Some(description) = site.description.as_ref() {
        p { (description) }
      }

      a href=(format!("https://{}", site.domain)) rel="noopener" { "Visit Site" }
    }
    .into_response()
}
