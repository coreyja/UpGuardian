use axum::{routing::{get, post}, Router};

use crate::app_state::AppState;

mod home;
mod login;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home::show))
        .route("/login", get(login::show))
        .route("/login/callback", get(login::callback))
        .route("/logout", post(login::logout))
}
