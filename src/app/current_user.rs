use leptos::*;
use leptos_query::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub user_id: String,
    pub session_id: String,
}

#[server]
pub async fn get_current_user(_args: ()) -> Result<Option<CurrentUser>, ServerFnError> {
    use crate::extractors::*;

    let session = extract_session()?;

    match session {
        Some(session) => Ok(Some(CurrentUser {
            user_id: session.user_id.to_string(),
            session_id: session.session_id.to_string(),
        })),
        None => Ok(None),
    }
}

pub fn use_current_user() -> QueryResult<Result<Option<CurrentUser>, ServerFnError>, impl RefetchFn>
{
    leptos_query::use_query(|| (), get_current_user, Default::default())
}
