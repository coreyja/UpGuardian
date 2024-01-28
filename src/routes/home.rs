use axum::extract::State;
use cja::server::session::DBSession;

use crate::{
    app_state::AppState,
    templates::{IntoTemplate, Template},
};

pub async fn show(session: Option<DBSession>, State(state): State<AppState>) -> Template {
    maud::html! {
        html {
            head {
                script src="https://unpkg.com/htmx.org@1.9.10" {}
                link rel="stylesheet" href="/styles/tailwind.css" {}

                title { "Status - Uptime Monitoring by coreyja" }
            }

            body {
                @if let Some(session) = session.clone() {
                    h1 { "Hello, " (session.user_id) "!" }
                    h3 { "Session Id: " (session.session_id) }

                    form method="POST" action="/logout" {
                        button type="submit" { "Logout" }
                    }

                    a href="/my/sites" { "My Sites" }
                } @else {
                    h1 { "Hello, world!" }

                    p { "You are not logged in!" }
                    p { "Click " a href="/login" { "here" } " to login!" }
                }

                p { "This is a Uptime Tracker built by "
                    a href="https://coreyja.com" { "coreyja!" }
                }
            }
        }
    }
    .into_template(state, session)
    .await
    .unwrap()
}
