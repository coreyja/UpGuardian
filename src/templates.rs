use axum::{http::HeaderMap, response::IntoResponse};
use cja::{app_state::AppState as _, server::session::DBSession};
use maud::{html, Markup, Render};
use miette::IntoDiagnostic;

use crate::app_state::AppState;

pub struct Template {
    pub content: Markup,
    sites: Vec<SidebarSiteLink>,
    session: Option<DBSession>,
}

impl IntoResponse for Template {
    fn into_response(self) -> axum::response::Response {
        {
            let outer_html = html! {
                html class="h-full bg-white" {
                  head {
                    link rel="stylesheet" href="/styles/tailwind.css" {}

                    title { "Status - Uptime Monitoring by coreyja" }
                  }

                  body class="h-full" {
                    div {
                      div."relative z-50 lg:hidden" role="dialog" aria-modal="true" {
                          div."fixed inset-0 bg-gray-900/80" {}
                          div."fixed inset-0 flex" {
                              div."relative mr-16 flex w-full max-w-xs flex-1" {
                                  div."absolute left-full top-0 flex w-16 justify-center pt-5" {
                                      button."-m-2.5 p-2.5" type="button" {
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
                                          img."h-8 w-auto" src="https://tailwindui.com/img/logos/mark.svg?color=white" alt="Your Company" {}
                                      }
                                      nav."flex flex-1 flex-col" {
                                          ul."flex flex-1 flex-col gap-y-7" role="list" {
                                              li {
                                                  ul."-mx-2 space-y-1" role="list" {
                                                      li {
                                                          a."bg-indigo-700 text-white group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              svg."h-6 w-6 shrink-0 text-white" stroke-width="1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true" {
                                                                  path stroke-linejoin="round" stroke-linecap="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" {}
                                                              }
                                                              "Dashboard"
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              svg."h-6 w-6 shrink-0 text-indigo-200 group-hover:text-white" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5" fill="none" aria-hidden="true" {
                                                                  path stroke-linecap="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" stroke-linejoin="round" {}
                                                              }
                                                              "Team"
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              svg."h-6 w-6 shrink-0 text-indigo-200 group-hover:text-white" aria-hidden="true" fill="none" stroke-width="1.5" viewBox="0 0 24 24" stroke="currentColor" {
                                                                  path stroke-linejoin="round" stroke-linecap="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" {}
                                                              }
                                                              "Projects"
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              svg."h-6 w-6 shrink-0 text-indigo-200 group-hover:text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true" stroke-width="1.5" {
                                                                  path stroke-linecap="round" d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5" stroke-linejoin="round" {}
                                                              }
                                                              "Calendar"
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              svg."h-6 w-6 shrink-0 text-indigo-200 group-hover:text-white" stroke-width="1.5" stroke="currentColor" aria-hidden="true" viewBox="0 0 24 24" fill="none" {
                                                                  path stroke-linecap="round" stroke-linejoin="round" d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 011.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 00-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 01-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 00-3.375-3.375h-1.5a1.125 1.125 0 01-1.125-1.125v-1.5a3.375 3.375 0 00-3.375-3.375H9.75" {}
                                                              }
                                                              "Documents"
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              svg."h-6 w-6 shrink-0 text-indigo-200 group-hover:text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5" aria-hidden="true" {
                                                                  path stroke-linecap="round" stroke-linejoin="round" d="M10.5 6a7.5 7.5 0 107.5 7.5h-7.5V6z" {}
                                                                  path stroke-linejoin="round" d="M13.5 10.5H21A7.5 7.5 0 0013.5 3v7.5z" stroke-linecap="round" {}
                                                              }
                                                              "Reports"
                                                          }
                                                      }
                                                  }
                                              }
                                              li {
                                                  div."text-xs font-semibold leading-6 text-indigo-200" {
                                                      "Your teams"
                                                  }
                                                  ul."-mx-2 mt-2 space-y-1" role="list" {
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              span."flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-indigo-400 bg-indigo-500 text-[0.625rem] font-medium text-white" {
                                                                  "H"
                                                              }
                                                              span."truncate" {
                                                                  "Heroicons"
                                                              }
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              span."flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-indigo-400 bg-indigo-500 text-[0.625rem] font-medium text-white" {
                                                                  "T"
                                                              }
                                                              span."truncate" {
                                                                  "Tailwind Labs"
                                                              }
                                                          }
                                                      }
                                                      li {
                                                          a."text-indigo-200 hover:text-white hover:bg-indigo-700 group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href="#" {
                                                              span."flex h-6 w-6 shrink-0 items-center justify-center rounded-lg border border-indigo-400 bg-indigo-500 text-[0.625rem] font-medium text-white" {
                                                                  "W"
                                                              }
                                                              span."truncate" {
                                                                  "Workcation"
                                                              }
                                                          }
                                                      }
                                                  }
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
                              div."flex h-16 shrink-0 items-center" {
                                  // img."h-8 w-auto" src="https://tailwindui.com/img/logos/mark.svg?color=white" alt="Your Company" {}
                                  a href="/" {
                                    h1."text-white text-2xl" { "Status" }
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
                                              },
                                              SideBarLink {
                                                text: "Sites".to_string(),
                                                href: "/my/sites".to_string(),
                                                selected: false,
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
                          button."-m-2.5 p-2.5 text-indigo-200 lg:hidden" type="button" {
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
          a.(self.conditional_selected_styles())." group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold" href=(self.href) {
            svg."h-6 w-6 shrink-0 text-white" stroke="currentColor" aria-hidden="true" fill="none" stroke-width="1.5" viewBox="0 0 24 24" {
                path stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" stroke-linecap="round" {}
            }
            (self.text)
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
        })
    }
}
