use axum::{http::HeaderMap, response::IntoResponse};
use maud::{html, Markup};

pub struct Template {
    pub content: Markup,
}

impl IntoResponse for Template {
    fn into_response(self) -> axum::response::Response {
        {
            let this = html! {
              head {
                link rel="stylesheet" href="/styles/tailwind.css" {}

                title { "Status - Uptime Monitoring by coreyja" }
              }

              body {
                (self.content)
              }
            };

            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("text/html; charset=utf-8"),
            );

            (headers, this.0).into_response()
        }
    }
}

pub trait IntoTemplate {
    fn into_template(self) -> Template;
}

impl IntoTemplate for Markup {
    fn into_template(self) -> Template {
        Template { content: self }
    }
}
