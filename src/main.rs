use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use serde_json::Value;
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

    let app = Router::new()
        .route("/", get(root))
        .route(
            "/login",
            get(|| async move {
                let idp_url = std::env::var("COREYJA_IDP_URL")
                    .unwrap_or_else(|_| "http://localhost:3000".into());
                let login_url = format!("{}/login/status", idp_url);

                Redirect::temporary(&login_url)
            }),
        )
        .route("/login/callback", get(login_callback));

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

#[derive(Debug, Deserialize)]
struct LoginCallback {
    state: String,
}

async fn login_callback(Query(query): Query<LoginCallback>) -> impl IntoResponse {
    let idp_url =
        std::env::var("COREYJA_IDP_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/login/claim/{}", idp_url, query.state))
        .send()
        .await
        .into_diagnostic()
        .unwrap();

    let json = resp.json::<Value>().await.unwrap();

    Json(json)
}
