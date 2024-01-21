use axum::response::IntoResponse;
use cja::server::session::DBSession;

pub async fn show(session: Option<DBSession>) -> impl IntoResponse {
    maud::html! {
        html {
            head {
                script src="https://unpkg.com/htmx.org@1.9.10" {}

                title { "Status - Uptime Monitoring by coreyja" }
            }

            body {
                @if let Some(session) = session {
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
}
