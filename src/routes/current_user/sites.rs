use axum::{
    extract::{FromRequestParts, Path, State},
    http::{request::Parts},
    response::{IntoResponse, Redirect},
    Form,
};

use cja::{
    app_state::AppState as _,
    server::session::{DBSession, SessionRedirect},
};
use maud::html;
use serde::Deserialize as _;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::routes::current_user::pages::Page;

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

pub struct Site {
    pub site_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub domain: String,
    pub description: Option<String>,
}

#[derive(serde::Deserialize)]
struct SiteParams {
    site_id: Uuid,
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for Site {
    type Rejection = SessionRedirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = DBSession::from_request_parts(parts, state).await?;

        let Path(SiteParams { site_id }) =
            axum::extract::Path::<SiteParams>::from_request_parts(parts, state)
                .await
                .unwrap();

        let site = sqlx::query_as!(
            Site,
            r#"
          SELECT *
          FROM Sites
          WHERE site_id = $1 AND user_id = $2
        "#,
            site_id,
            session.user_id
        )
        .fetch_optional(state.db())
        .await
        .unwrap();

        let site = match site {
            Some(site) => site,
            None => panic!("TODO: 404"),
        };

        Ok(site)
    }
}

pub async fn show(site: Site, State(state): State<AppState>) -> axum::response::Response {
    let pages = sqlx::query_as!(
        Page,
        r#"
      SELECT *
      FROM Pages
      WHERE site_id = $1
    "#,
        site.site_id,
    )
    .fetch_all(state.db())
    .await
    .unwrap();

    html! {
      h1 { (site.name) }

      @if let Some(description) = site.description.as_ref() {
        p { (description) }
      }

      a href=(format!("https://{}", site.domain)) rel="noopener" { "Visit Site" }

      h2 { "Pages" }

      a href=(format!("/my/sites/{}/pages/new", site.site_id)) { "Create a new page" }

      ul {
        @for page in pages {
          li {
            a href=(format!("/my/sites/{}/pages/{}", site.site_id, page.page_id)) {
              (page.name) " - " (page.path)
            }
          }
        }
      }
    }
    .into_response()
}
