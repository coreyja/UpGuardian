use cja::server::cookies::CookieKey;
use miette::{miette, IntoDiagnostic};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: PgPool,
    cookie_key: CookieKey,
    pub font_awesome_kit_id: String,
}

impl AppState {
    pub async fn from_env() -> miette::Result<Self> {
        let pool = crate::setup::setup_db_pool().await?;
        let cookie_key = CookieKey::from_env_or_generate().into_diagnostic()?;
        let font_awesome_kit_id = std::env::var("FONT_AWESOME_KIT_ID")
            .map_err(|_| miette!("FONT_AWESOME_KIT_ID must be set"))?;

        Ok(Self {
            pool,
            cookie_key,
            font_awesome_kit_id,
        })
    }
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
