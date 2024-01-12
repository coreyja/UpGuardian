use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use cja::{
    app_state::AppState as _,
    cron::{CronRegistry, Worker},
    jobs::{worker::job_worker, Job},
    server::{cookies::CookieKey, session::DBSession},
    tower_cookies::CookieManagerLayer,
};
use jobs::hello::Hello;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use setup::setup_sentry;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::{net::TcpListener, task::JoinError};
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

#[tracing::instrument(err)]
pub async fn setup_db_pool() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .into_diagnostic()?;

    const MIGRATION_LOCK_ID: i64 = 0xDB_DB_DB_DB_DB_DB_DB;
    sqlx::query!("SELECT pg_advisory_lock($1)", MIGRATION_LOCK_ID)
        .execute(&pool)
        .await
        .into_diagnostic()?;

    sqlx::migrate!().run(&pool).await.into_diagnostic()?;

    let unlock_result = sqlx::query!("SELECT pg_advisory_unlock($1)", MIGRATION_LOCK_ID)
        .fetch_one(&pool)
        .await
        .into_diagnostic()?
        .pg_advisory_unlock;

    match unlock_result {
        Some(b) => {
            if b {
                tracing::info!("Migration lock unlocked");
            } else {
                tracing::info!("Failed to unlock migration lock");
            }
        }
        None => panic!("Failed to unlock migration lock"),
    }

    Ok(pool)
}

async fn run_axum(app_state: AppState) -> miette::Result<()> {
    let app = Router::new()
        .route("/", get(root))
        .route("/login", get(login))
        .route("/login/callback", get(login_callback))
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

async fn login(session: Option<DBSession>) -> Response {
    if session.is_none() {
        let idp_url =
            std::env::var("COREYJA_IDP_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let login_url = format!("{}/login/status", idp_url);
        Redirect::temporary(&login_url).into_response()
    } else {
        Redirect::temporary("/").into_response()
    }
}

async fn _main() -> Result<()> {
    setup::setup_tracing()?;

    tracing::info!("Hello, world!");

    let pool = setup_db_pool().await?;
    let app_state = AppState {
        pool,
        cookie_key: CookieKey::from_env_or_generate().into_diagnostic()?,
    };

    cja::sqlx::migrate!()
        .run(app_state.db())
        .await
        .into_diagnostic()?;

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
struct AppState {
    pool: PgPool,
    cookie_key: CookieKey,
}

impl cja::app_state::AppState for AppState {
    fn version(&self) -> &str {
        "dev"
    }

    fn db(&self) -> &cja::sqlx::PgPool {
        &self.pool
    }

    fn cookie_key(&self) -> &CookieKey {
        &self.cookie_key
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

async fn login_callback(
    cookies: tower_cookies::Cookies,
    Query(query): Query<LoginCallback>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
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

    #[derive(Debug, Serialize, Deserialize)]
    struct ClaimResponse {
        user_id: String,
    }
    let json = resp.json::<ClaimResponse>().await.unwrap();

    let user = sqlx::query!(
        "INSERT INTO Users (user_id, coreyja_user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING *",
        uuid::Uuid::new_v4(),
        uuid::Uuid::parse_str(&json.user_id).unwrap(),
    )
    .fetch_one(app_state.db())
    .await
    .unwrap();

    DBSession::create(user.user_id, &app_state, &cookies)
        .await
        .unwrap();

    Redirect::temporary("/").into_response()
}
