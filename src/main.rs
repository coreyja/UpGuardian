use std::net::SocketAddr;

use cja::{app_state::AppState as _, jobs::worker::job_worker, tower_cookies::CookieManagerLayer};
use miette::{Context, IntoDiagnostic, Result};
use setup::setup_sentry;
use tokio::{net::TcpListener, task::JoinError};
use tracing::info;

use crate::{jobs::Jobs, routes::routes};

mod app_state;
mod setup;

mod cron;
mod jobs;
mod routes;

use app_state::AppState;

fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .into_diagnostic()?
        .block_on(async { _main().await })
}

async fn run_axum(app_state: AppState) -> miette::Result<()> {
    let app = routes()
        .with_state(app_state)
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app)
        .await
        .into_diagnostic()
        .wrap_err("Failed to run server")?;

    Ok(())
}

async fn _main() -> Result<()> {
    setup::setup_tracing()?;

    tracing::info!("Hello, world!");

    let app_state = AppState::from_env().await?;

    cja::sqlx::migrate!()
        .run(app_state.db())
        .await
        .into_diagnostic()?;

    info!("Spawning Tasks");
    let futures = vec![
        tokio::spawn(run_axum(app_state.clone())),
        tokio::spawn(job_worker(app_state.clone(), Jobs)),
        tokio::spawn(cron::run_cron(app_state.clone())),
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
