use std::{fmt::Display, ops::Neg};

use axum::{
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, response},
    response::{IntoResponse, Redirect},
    Form,
};

use chrono::Duration;
use cja::{
    app_state::AppState as _,
    server::session::{DBSession, SessionRedirect},
};
use maud::{html, Render};
use serde::Deserialize as _;
use sqlx::postgres::types::PgInterval;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    routes::current_user::pages::{self, Checkin},
    templates::IntoTemplate,
};
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

      (site_stats_overview(&site, &pages, &state).await)

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

struct Change {
    value: String,
    diff: Diff,
    emotion: Emotion,
}

enum Diff {
    Increase,
    Decrease,
}

enum Emotion {
    Happy,
    Sad,
}

impl Change {
    fn icon_class(&self) -> &'static str {
        match self.diff {
            Diff::Increase => "fa-arrow-up",
            Diff::Decrease => "fa-arrow-down",
        }
    }

    fn screen_reader_text(&self) -> &'static str {
        match self.diff {
            Diff::Increase => "Increased by",
            Diff::Decrease => "Decreased by",
        }
    }
}

impl Emotion {
    fn color_class(&self) -> &'static str {
        match self {
            Emotion::Happy => "text-green-600",
            Emotion::Sad => "text-red-600",
        }
    }
}

#[derive(Copy, Clone)]
enum IconStyle {
    Solid,
    Regular,
    Light,
}

impl Display for IconStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconStyle::Solid => write!(f, "fa-solid"),
            IconStyle::Regular => write!(f, "fa-regular"),
            IconStyle::Light => write!(f, "fa-light"),
        }
    }
}

fn icon(icon_class: &str, style: IconStyle) -> maud::Markup {
    html! {
      i class=(format!("{icon_class} {style}")) {}
    }
}

fn single_stat<S: std::fmt::Display>(
    title: &str,
    stat: S,
    change: Option<Change>,
    icon_class: &str,
) -> maud::Markup {
    html! {
      div."relative overflow-hidden rounded-lg bg-white px-4 py-5 shadow sm:px-6 sm:pt-6" {
        dt {
            div."absolute rounded-md bg-indigo-500 p-3" {
                (icon(&format!("text-white fa-fw {icon_class}"), IconStyle::Solid))
            }
            p."ml-16 truncate text-sm font-medium text-gray-500" {
                (title)
            }
        }
        dd."ml-16 flex items-baseline" {
            p."text-2xl font-semibold text-gray-900" {
              (stat)
            }
            @if let Some(change) = change {
              (change)
            }
        }
      }
    }
}

impl Render for Change {
    fn render(&self) -> maud::Markup {
        html! {
            p class=(format!("ml-2 flex items-baseline text-sm font-semibold {}", self.emotion.color_class())) {
                (icon(&format!("h-5 w-5 fa-fw {}", self.icon_class()), IconStyle::Solid))
                span."sr-only" {
                    (self.screen_reader_text())
                }
                (self.value)
            }
        }
    }
}

fn chrono_to_pg_interval(duration: Duration) -> PgInterval {
    let std = duration.to_std().unwrap();
    std.try_into().unwrap()
}

async fn site_stats_overview(site: &Site, pages: &[Page], state: &AppState) -> maud::Markup {
    let pages_tracked = pages.len();

    let recent_duration = chrono::Duration::days(30);

    let all_interval = chrono_to_pg_interval(recent_duration * 2);
    let all_checkins = sqlx::query_as!(
        Checkin,
        r#"
    SELECT Checkins.*
    FROM Checkins
    JOIN Pages using (page_id)
    WHERE Pages.site_id = $1 AND
          now() - Checkins.created_at < $2
    ORDER BY Checkins.created_at DESC
  "#,
        site.site_id,
        all_interval
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

    let response_time_change = if old_checkins.is_empty() {
        None
    } else {
        let old_response_time = avg_response_time_for_checkins(&old_checkins);

        let response_time_change = old_response_time - avg_response_time;
        let response_time_change = response_time_change / old_response_time * 100.0;

        let response_time_abs_change = response_time_change.abs();
        let response_time_formatted = format!("{:.1}%", response_time_abs_change);

        if response_time_change.abs() < 0.00001 {
            None
        } else {
            let (diff, emotion) = if response_time_change.is_sign_positive() {
                (Diff::Increase, Emotion::Sad)
            } else {
                (Diff::Decrease, Emotion::Happy)
            };
            let response_time_change = Change {
                value: response_time_formatted,
                diff,
                emotion,
            };

            Some(response_time_change)
        }
    };

    let avg_response_time = format!("{:.1} ms", avg_response_time);

    let succesful_percent = success_percent_for_checkins(&new_checkins);
    let successful_percent_change = if old_checkins.is_empty() || new_checkins.is_empty() {
        None
    } else {
        let old_succesful_percent = success_percent_for_checkins(&old_checkins);

        let succesful_percent_change = old_succesful_percent - succesful_percent;
        if succesful_percent_change.abs() < 0.00001 {
            None
        } else {
            let succesful_change_formatted = format!("{:.1}%", succesful_percent_change.abs());
            let (diff, emotion) = if succesful_percent_change.is_sign_positive() {
                (Diff::Increase, Emotion::Happy)
            } else {
                (Diff::Decrease, Emotion::Sad)
            };
            let successful_percent_change = Change {
                value: succesful_change_formatted,
                diff,
                emotion,
            };
            Some(successful_percent_change)
        }
    };
    let succesful_percent = format!("{:.1}%", succesful_percent);

    html! {
        div {
            h3."text-base font-semibold leading-6 text-gray-900" {
                "Last 30 days"
            }
            (new_checkins.len())
            dl."mt-5 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3" {
                (
                  single_stat("Pages Tracked", pages_tracked, None, "fa-file")
                )
                (
                  single_stat("Avg. Response Time", avg_response_time, response_time_change, "fa-clock")
                )
                (
                  single_stat("Success Rate", succesful_percent, successful_percent_change, "fa-check")
                )
            }
        }
    }
}

fn success_percent_for_checkins(checkins: &[Checkin]) -> f64 {
    let count_successful = checkins
        .iter()
        .filter(|checkin| checkin.outcome == "success")
        .count();

    count_successful as f64 / checkins.len() as f64 * 100.0
}

fn avg_response_time_for_checkins(checkins: &[Checkin]) -> f64 {
    let total_nanos = checkins
        .iter()
        .filter_map(|checkin| checkin.duration_nanos)
        .sum::<i64>();
    let total_millis = total_nanos as f64 / 1_000_000.0;

    total_millis / checkins.len() as f64
}
