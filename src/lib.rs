pub mod app;
pub mod error_template;

#[cfg(feature = "ssr")]
pub mod fileserv;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

#[cfg(feature = "ssr")]
pub mod app_state;
#[cfg(feature = "ssr")]
pub mod setup;

#[cfg(feature = "ssr")]
pub mod cron;
#[cfg(feature = "ssr")]
pub mod jobs;
#[cfg(feature = "ssr")]
pub mod routes;

#[cfg(feature = "ssr")]
pub mod templates;

#[cfg(feature = "ssr")]
pub use app_state::AppState;

#[cfg(feature = "ssr")]
pub mod extractors;
