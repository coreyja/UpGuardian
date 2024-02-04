#[cfg(feature = "ssr")]
fn main() -> miette::Result<()> {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use status::app::*;
    use status::fileserv::file_and_error_handler;

    use std::net::SocketAddr;

    use cja::{
        app_state::AppState as _, jobs::worker::job_worker, tower_cookies::CookieManagerLayer,
    };
    use miette::{Context, IntoDiagnostic, Result};
    use status::setup::setup_sentry;
    use tokio::{net::TcpListener, task::JoinError};
    use tracing::info;

    use status::{jobs::Jobs, routes::routes};


    use status::app_state::AppState;

    async fn run_axum(app_state: AppState) -> miette::Result<()> {
        // let app = routes().with_state(app_state);

        // let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
        // let listener = TcpListener::bind(&addr).await.unwrap();
        // tracing::debug!("listening on {}", addr);

        // axum::serve(listener, app)
        //     .await
        //     .into_diagnostic()
        //     .wrap_err("Failed to run server")?;

        // Setting get_configuration(None) means we'll be using cargo-leptos's env values
        // For deployment these variables are:
        // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
        // Alternately a file can be specified such as Some("Cargo.toml")
        // The file would need to be included with the executable when moved to deployment
        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(App);

        // build our application with a route
        let app = Router::new()
            .leptos_routes(&leptos_options, routes, App)
            .fallback(file_and_error_handler)
            .with_state(leptos_options)
            .layer(CookieManagerLayer::new());

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        logging::log!("listening on http://{}", &addr);
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }

    async fn _main() -> Result<()> {
        status::setup::setup_tracing()?;

        tracing::info!("Hello, world!");

        let app_state = AppState::from_env().await?;

        cja::sqlx::migrate!()
            .run(app_state.db())
            .await
            .into_diagnostic()?;

        info!("Spawning Tasks");
        let futures = vec![
            tokio::spawn(run_axum(app_state.clone())),
            // tokio::spawn(job_worker(app_state.clone(), Jobs)),
            // tokio::spawn(status::cron::run_cron(app_state.clone())),
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

    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .into_diagnostic()?
        .block_on(async { _main().await })
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
