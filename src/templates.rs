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

                    script src="/public/frontend/index.js" {}

                    meta name="viewport" content="width=device-width, initial-scale=1.0" {}

                    title { "UpGuardian - Uptime Monitoring by coreyja" }
                  }

                  body class="h-full" {
                    div {
                      div."relative z-50 lg:hidden hidden" role="dialog" aria-modal="true" {
                          div."fixed inset-0 bg-gray-900/80" {}
                          div."fixed inset-0 flex" {
                              div."relative mr-16 flex w-full max-w-xs flex-1" {
                                  div."absolute left-full top-0 flex w-16 justify-center pt-5" {
                                      button."-m-2.5 p-2.5" type="button" data-app="ToggleClass" data-class="hidden" data-target="div[role='dialog']" {
                                          span."sr-only" {
                                              "Close sidebar"
                                          }
                                          svg."h-6 w-6 text-white" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true" viewBox="0 0 24 24" {
                                              path stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" stroke-linecap="round" {}
                                          }
                                      }
                                  }
                                  div."flex grow flex-col gap-y-5 overflow-y-auto bg-indigo-600 px-6 pb-2" {
                                      div."flex h-16 shrink-0 items-center" {
                                        a href="/" {
                                          img."h-12 w-auto mt-2" src="/public/Logomark.png" alt="UpGuardian by Coreyja" {}
                                        }
                                      }
                                      nav."flex flex-1 flex-col" {
                                          ul."flex flex-1 flex-col gap-y-7" role="list" {
                                              li {
                                                (SideBarLinkList {
                                                    links: vec![
                                                      SideBarLink {
                                                        text: "Dashboard".to_string(),
                                                        href: "/".to_string(),
                                                        selected: false,
                                                        icon: "fa-solid fa-house".to_string(),
                                                      },
                                                      SideBarLink {
                                                        text: "Sites".to_string(),
                                                        href: "/my/sites".to_string(),
                                                        selected: false,
                                                        icon: "fa-solid fa-globe".to_string(),
                                                      },
                                                    ]
                                                  })
                                              }
                                              li {
                                                  div."text-xs font-semibold leading-6 text-indigo-200" {
                                                      "Your sites"
                                                  }

                                                  (SidebarSiteList {
                                                    sites: self.sites.clone()
                                                  })
                                              }
                                          }
                                      }
                                  }
                              }
                          }
                      }

                      // This is the sidebar for larger screens
                      div."hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col" {
                          div."flex grow flex-col gap-y-5 overflow-y-auto bg-indigo-600 px-6" {
                              div."flex shrink-0 items-center" {
                                  a href="/" {
                                    // h1."text-white text-2xl" { "UpGuardian" }
                                    img."w-auto mt-4" src="/public/Logo.png" alt="UpGuradian by coreyja" {}
                                  }
                              }
                              nav."flex flex-1 flex-col" {
                                  ul."flex flex-1 flex-col gap-y-7" role="list" {
                                      li {
                                          (SideBarLinkList {
                                            links: vec![
                                              SideBarLink {
                                                text: "Dashboard".to_string(),
                                                href: "/".to_string(),
                                                selected: false,
                                                icon: "fa-solid fa-house".to_string(),
                                              },
                                              SideBarLink {
                                                text: "Sites".to_string(),
                                                href: "/my/sites".to_string(),
                                                selected: false,
                                                icon: "fa-solid fa-globe".to_string(),
                                              },
                                            ]
                                          })
                                      }
                                      li {
                                          div."text-xs font-semibold leading-6 text-indigo-200" {
                                              "Your Sites"
                                          }
                                          (SidebarSiteList {
                                            sites: self.sites
                                          })
                                      }
                                      li."-mx-6 mt-auto" {
                                          @if self.session.is_some() {
                                            a."flex items-center gap-x-4 px-6 py-3 text-sm font-semibold leading-6 text-white hover:bg-indigo-700" href="#" {
                                              // img."h-8 w-8 rounded-full bg-indigo-700" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" {}
                                              span {
                                                  "You are logged in"
                                              }
                                            }
                                          } @else {
                                            a."flex items-center gap-x-4 px-6 py-3 text-sm font-semibold leading-6 text-white hover:bg-indigo-700" href="/login" {
                                              // img."h-8 w-8 rounded-full bg-indigo-700" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" {}
                                              span {
                                                  "Login"
                                              }
                                            }
                                          }
                                      }
                                  }
                              }
                          }
                      }
                      div."sticky top-0 z-40 flex items-center gap-x-6 bg-indigo-600 px-4 py-4 shadow-sm sm:px-6 lg:hidden" {
                          button."-m-2.5 p-2.5 text-indigo-200 lg:hidden" type="button" data-app="ToggleClass" data-class="hidden" data-target="div[role='dialog']"{
                              span."sr-only" {
                                  "Open sidebar"
                              }
                              svg."h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true" stroke-width="1.5" {
                                  path stroke-linecap="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" stroke-linejoin="round" {}
                              }
                          }
                          div."flex-1 text-sm font-semibold leading-6 text-white" {
                              "Dashboard"
                          }
                          a href="#" {
                              span."sr-only" {
                                  "Your profile"
                              }
                              img."h-8 w-8 rounded-full bg-indigo-700" src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80" alt="" {}
                          }
                      }
                      main."py-10 lg:pl-72" {
                          div."px-4 sm:px-6 lg:px-8" {
                            (self.content)
                          }
                      }
                  }
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

#[derive(Debug, Clone)]
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
