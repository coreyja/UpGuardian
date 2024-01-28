use axum::{
    extract::{FromRequestParts, Path, State},
    http::request::Parts,
    response::{IntoResponse, Redirect},
    Form,
};

use cja::{
    app_state::AppState as _,
    server::session::{DBSession, SessionRedirect},
};
use maud::{html, Render};
use serde::Deserialize as _;
use uuid::Uuid;

use crate::{app_state::AppState, templates::IntoTemplate};
use crate::{routes::current_user::pages::Page, templates::Template};

struct SiteTableRow {
    site_id: Uuid,
    name: String,
    domain: String,
    description: Option<String>,
}

impl Render for SiteTableRow {
    fn render(&self) -> maud::Markup {
        html! {
          div."min-w-0" {
            div."flex items-start gap-x-3" {
                p."text-sm font-semibold leading-6 text-gray-900" {
                    // "GraphQL API"
                    a href=(format!("/my/sites/{}", self.site_id)) {
                        (self.name)
                    }
                }
                // TODO: Check the last checkin for the 'status'
                p."rounded-md whitespace-nowrap mt-0.5 px-1.5 py-0.5 text-xs font-medium ring-1 ring-inset text-green-700 bg-green-50 ring-green-600/20" {
                    "Complete"
                }
            }
            @if let Some(description) = self.description.as_ref() {
              div."mt-1 flex items-center gap-x-2 text-xs leading-5 text-gray-500" {
                p."truncate" {
                  (description)
                }
              }
            }
            div."mt-1 flex items-center gap-x-2 text-xs leading-5 text-gray-500" {
                p."whitespace-nowrap" {
                    "Last Checked at "
                    // TODO: Grab the last checked at
                    time datetime="2023-03-17T00:00Z" {
                        "March 17, 2023"
                    }
                }
                svg."h-0.5 w-0.5 fill-current" viewBox="0 0 2 2" {
                    circle cy="1" cx="1" r="1" {}
                }
                p."truncate" {
                    (self.domain)
                }
            }
        }
        div."flex flex-none items-center gap-x-4" {
            a."hidden rounded-md bg-white px-2.5 py-1.5 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 sm:block" href=(format!("/my/sites/{}", self.site_id)) {
                "View project"
                span."sr-only" {
                    ", " (self.name)
                }
            }
            div."relative flex-none" {
                button."-m-2.5 block p-2.5 text-gray-500 hover:text-gray-900" id="options-menu-0-button" aria-expanded="false" type="button" aria-haspopup="true" {
                    span."sr-only" {
                        "Open options"
                    }
                    svg."h-5 w-5" viewBox="0 0 20 20" aria-hidden="true" fill="currentColor" {
                        path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z" {}
                    }
                }
                div."absolute right-0 z-10 mt-2 w-32 origin-top-right rounded-md bg-white py-2 shadow-lg ring-1 ring-gray-900/5 focus:outline-none hidden" tabindex="-1" aria-orientation="vertical" aria-labelledby="options-menu-0-button" role="menu" {
                    a."block px-3 py-1 text-sm leading-6 text-gray-900" #options-menu-0-item-0 role="menuitem" tabindex="-1" href="#" {
                        "Edit"
                        span."sr-only" {
                          ", " (self.name)
                        }
                    }
                    a."block px-3 py-1 text-sm leading-6 text-gray-900" #options-menu-0-item-1 role="menuitem" tabindex="-1" href="#" {
                        "Move"
                        span."sr-only" {
                          ", " (self.name)
                        }
                    }
                    a."block px-3 py-1 text-sm leading-6 text-gray-900" #options-menu-0-item-2 role="menuitem" tabindex="-1" href="#" {
                        "Delete"
                        span."sr-only" {
                          ", " (self.name)
                        }
                    }
                }
            }
        }
        }
    }
}

pub async fn index(session: DBSession, State(state): State<AppState>) -> impl IntoResponse {
    let sites = sqlx::query_as!(
        SiteTableRow,
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

    let table = html! {
        ul."divide-y divide-gray-100" role="list" {
          @for site in sites {
            li."flex items-center justify-between gap-x-6 py-5" {
              (site)
            }
          }
        }
    };

    html! {
      h1 { "My Sites" }

      a href="/my/sites/new" { "Create a new site" }

      (table)
    }
    .into_template(state, Some(session))
    .await
    .unwrap()
}

pub async fn new(session: DBSession, State(state): State<AppState>) -> impl IntoResponse {
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
    .into_template(state, Some(session))
    .await
    .unwrap()
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

pub async fn show(site: Site, State(state): State<AppState>, session: DBSession) -> Template {
    let pages = sqlx::query_as!(
        Page,
        r#"
      SELECT Pages.*
      FROM Pages
      JOIN Sites ON Sites.site_id = Pages.site_id
      WHERE Pages.site_id = $1 AND Sites.user_id = $2
    "#,
        site.site_id,
        session.user_id
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
    .into_template(state, Some(session))
    .await
    .unwrap()
}
