use leptos::*;
use serde::{Deserialize, Serialize};

pub mod index;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub site_id: uuid::Uuid,
    pub name: String,
    pub domain: String,
    pub description: Option<String>,
}

#[server]
pub async fn get_my_sites(limit: Option<i64>) -> Result<Vec<Site>, ServerFnError> {
    use crate::extractors::*;
    use cja::app_state::AppState as _;

    let session = extract_session()?;
    let state = extract_state()?;

    let sites = if let Some(session) = &session {
        sqlx::query_as!(
            Site,
            r#"
        SELECT site_id, name, domain, description
        FROM Sites
        WHERE user_id = $1
        LIMIT $2
      "#,
            session.user_id,
            limit.unwrap_or(i64::MAX)
        )
        .fetch_all(state.db())
        .await
        .unwrap()
    } else {
        vec![]
    };

    Ok(sites)
}

impl Site {
    pub fn href(&self) -> String {
        format!("/my/sites/{}", self.site_id)
    }
}
