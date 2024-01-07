use cja::jobs::Job;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Hello;

#[async_trait::async_trait]
impl Job<AppState> for Hello {
    async fn run(&self, _app_state: AppState) -> miette::Result<()> {
        tracing::info!("Hello, world!");

        Ok(())
    }

    const NAME: &'static str = "Hello";
}
