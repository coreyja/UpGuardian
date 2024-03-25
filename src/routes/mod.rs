use axum::{
    extract::Path,
    response::{IntoResponse as _, Response},
    routing::{get, post},
    Router,
};
use include_dir::Dir;
use miette::IntoDiagnostic as _;

use crate::app_state::AppState;

mod current_user;
mod home;
mod login;

const STATIC_ASSETS: Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/public");

async fn static_assets(Path(p): Path<String>) -> Response {
    let path = p.strip_prefix('/').unwrap_or(&p);
    let path = path.strip_suffix('/').unwrap_or(path);

    let entry = STATIC_ASSETS.get_file(path);

    let Some(entry) = entry else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            format!("Static asset {path} not found"),
        )
            .into_response();
    };

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        mime.to_string().parse().into_diagnostic().unwrap(),
    );

    (headers, entry.contents()).into_response()
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home::show))
        .route("/public/*path", get(static_assets))
        .route("/styles/tailwind.css", get(tailwind_css))
        .route("/login", get(login::show))
        .route("/login/callback", get(login::callback))
        .route("/logout", post(login::logout))
        .route(
            "/my/sites",
            get(current_user::sites::index).post(current_user::sites::create),
        )
        .route("/my/sites/new", get(current_user::sites::new))
        .route("/my/sites/:site_id", get(current_user::sites::show))
        .route(
            "/my/sites/:site_id/pages/new",
            get(current_user::pages::new),
        )
        .route(
            "/my/sites/:site_id/pages",
            post(current_user::pages::create),
        )
        .route(
            "/my/sites/:site_id/pages/:page_id",
            get(current_user::pages::show),
        )
        .route(
            "/my/sites/:site_id/pages/:page_id/refresh",
            get(current_user::pages::refresh),
        )
}

async fn tailwind_css() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/target/tailwind.css"))
}
