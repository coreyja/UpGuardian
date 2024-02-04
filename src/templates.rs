use axum::{http::HeaderMap, response::IntoResponse};
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::{html, Markup, Render};
use miette::IntoDiagnostic;

use crate::app_state::AppState;

pub struct Template {
    pub content: Markup,
    sites: Vec<SidebarSiteLink>,
    session: Option<DBSession>,
    state: AppState,
}

impl IntoResponse for Template {
    fn into_response(self) -> axum::response::Response {
        {
            let outer_html = html! {
                html class="h-full bg-white" {
                  head {
                    link rel="stylesheet" href="/styles/tailwind.css" {}
                    link rel="stylesheet" href=(format!("https://kit.fontawesome.com/{}.css", self.state.font_awesome_kit_id)) crossorigin="anonymous" {}

                    title { "Status - Uptime Monitoring by coreyja" }
                  }

                }

            };

            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
            );

            (headers, outer_html.0).into_response()
        }
    }
}

struct SideBarLinkList {
    links: Vec<SideBarLink>,
}

impl Render for SideBarLinkList {
    fn render(&self) -> Markup {
        html! {
          ul."-mx-2 space-y-1" role="list" {
            @for link in &self.links {
              li {
                (link.render())
              }
            }
          }
        }
    }
}

struct SideBarLink {
    text: String,
    href: String,
    selected: bool,
    icon: String,
}

impl SideBarLink {
    fn is_selected(&self) -> bool {
        self.selected
    }

    fn conditional_selected_styles(&self) -> String {
        if self.is_selected() {
            "bg-indigo-700 text-white".to_string()
        } else {
            "text-indigo-200 hover:text-white hover:bg-indigo-700".to_string()
        }
    }
}

impl Render for SideBarLink {
    fn render(&self) -> Markup {
        html! {
          a class=(format!("{} group rounded-md p-2 block flex gap-x-3", self.conditional_selected_styles())) href=(self.href) {
            i class=(format!("self-center shrink-0 text-white fa-fw {}", self.icon)) aria-hidden="true" {}

            span class="text-sm leading-6 font-semibold" {
                (self.text)
            }
          }
        }
    }
}

struct SidebarSiteList {
    sites: Vec<SidebarSiteLink>,
}

impl Render for SidebarSiteList {
    fn render(&self) -> Markup {
        html! {
          ul."-mx-2 space-y-1" role="list" {
            @for link in &self.sites {
              li {
                (link.render())
              }
            }
          }
        }
    }
}

struct SidebarSiteLink {
    name: String,
    href: String,
}

impl Render for SidebarSiteLink {
    fn render(&self) -> Markup {
        html! {
          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href=(self.href) {
            span."flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-indigo-400 bg-indigo-500 text-[0.625rem] font-medium text-white" {
              (self.name.chars().next().unwrap())
            }
            span."truncate" {
              (self.name)
            }
          }
        }
    }

    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.render().into_string());
    }
}

#[async_trait::async_trait]
pub trait IntoTemplate {
    async fn into_template(
        self,
        app_state: AppState,
        session: Option<DBSession>,
    ) -> miette::Result<Template>;
}

#[async_trait::async_trait]
impl IntoTemplate for Markup {
    async fn into_template(
        self,
        app_state: AppState,
        session: Option<DBSession>,
    ) -> miette::Result<Template> {
        let sites = if let Some(session) = &session {
            sqlx::query!(
                r#"
      SELECT Sites.*
      FROM Sites
      WHERE user_id = $1
      LIMIT 5
      "#,
                session.user_id
            )
            .fetch_all(app_state.db())
            .await
            .into_diagnostic()?
        } else {
            vec![]
        };

        let sites = sites
            .into_iter()
            .map(|site| SidebarSiteLink {
                name: site.name,
                href: format!("/my/sites/{}", site.site_id),
            })
            .collect();

        Ok(Template {
            content: self,
            sites,
            session,
            state: app_state,
        })
    }
}
