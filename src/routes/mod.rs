use axum::{routing::get, Router};

use crate::app_state::AppState;

mod home;
mod login;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home::show))
        .route("/login", get(login::show))
        .route("/login/callback", get(login::callback))
}
