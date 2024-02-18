use cja::server::session::DBSession;
use leptos::*;

use crate::AppState;

pub fn extract_session() -> Result<Option<DBSession>, ServerFnError> {
    use_context::<Option<DBSession>>()
        .ok_or_else(|| ServerFnError::ServerError("Auth session missing.".into()))
}
pub fn extract_state() -> Result<AppState, ServerFnError> {
    use_context::<AppState>().ok_or_else(|| ServerFnError::ServerError("App state missing.".into()))
}
