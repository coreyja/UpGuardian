use std::time::Duration;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use cja::{
    cron::{CronRegistry, Worker},
    jobs::{worker::job_worker, Job},
};
use jobs::hello::Hello;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use setup::setup_sentry;
use tokio::task::JoinError;
use tracing::info;

use crate::jobs::Jobs;

mod setup;

mod jobs;

fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .into_diagnostic()?
        .block_on(async { _main().await })
}

async fn run_axum(_app_state: AppState) -> miette::Result<()> {
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

async fn _main() -> Result<()> {
    setup::setup_tracing()?;

    tracing::info!("Hello, world!");

    let app_state = AppState;

    info!("Spawning Tasks");
    let futures = vec![
        tokio::spawn(run_axum(app_state.clone())),
        tokio::spawn(job_worker(app_state.clone(), Jobs)),
        tokio::spawn(run_cron(app_state.clone())),
    ];
    info!("Tasks Spawned");

    let results = futures::future::join_all(futures).await;
    let results: Result<Vec<Result<()>>, JoinError> = results.into_iter().collect();
    results
        .into_diagnostic()?
        .into_iter()
        .collect::<Result<Vec<()>>>()?;

    info!("Main Returning");

    Ok(())
}

#[derive(Debug, Clone)]
struct AppState;

impl cja::app_state::AppState for AppState {
    fn version(&self) -> &str {
        "dev"
    }

    fn db(&self) -> &cja::sqlx::PgPool {
        todo!()
    }
}

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register(
        "HelloWorld",
        Duration::from_secs(60),
        |app_state: AppState, context| Hello.enqueue(app_state.clone(), context),
    );

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> miette::Result<()> {
    Worker::new(app_state, cron_registry()).run().await?;

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

    #[derive(Debug, Serialize, Deserialize)]
    struct Claim {
        sub: String,
        exp: usize,
    }

    let key = std::env::var("AUTH_PRIVATE_KEY").unwrap();
    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::RS256),
        &Claim {
            sub: query.state,
            exp: (chrono::Utc::now() + chrono::Duration::minutes(1)).timestamp() as usize,
        },
        &EncodingKey::from_rsa_pem(key.as_bytes()).unwrap(),
    )
    .unwrap();

    let claim_url = format!("{}/login/status", idp_url);

    let resp = client
        .post(claim_url)
        .json(&json!({ "jwt": token }))
        .send()
        .await
        .into_diagnostic()
        .unwrap();

    let json = resp.json::<Value>().await.unwrap();

    Json(json)
}
