use axum::response::IntoResponse;

pub async fn show() -> impl IntoResponse {
    maud::html! {
        html {
            head {
                title { "Hello, world!" }
            }
            body {
                h1 { "Hello, world!" }
                p { "This is a simple web app written in Rust." }
            }
        }
    }
}
