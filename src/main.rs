use axum::{response::IntoResponse, routing::get, Router};
use miette::{IntoDiagnostic, Result};
use setup::setup_sentry;

mod setup;

fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .into_diagnostic()?
        .block_on(async { _main().await })
}

async fn _main() -> Result<()> {
    setup::setup_tracing()?;

    tracing::info!("Hello, world!");

    let app = Router::new().route("/", get(root));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3001".parse().into_diagnostic()?)
        .serve(app.into_make_service())
        .await
        .into_diagnostic()?;

    Ok(())
}

async fn root() -> impl IntoResponse {
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
