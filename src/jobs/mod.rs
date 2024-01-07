use cja::jobs::{registry::JobRegistry, worker::JobFromDB, Job};

use crate::AppState;

pub(crate) struct Jobs;

pub(crate) mod hello;

#[async_trait::async_trait]
impl JobRegistry<AppState> for Jobs {
    async fn run_job(&self, job: &JobFromDB, app_state: AppState) -> miette::Result<()> {
        let payload = job.payload.clone();

        match job.name.as_str() {
            "Hello" => hello::Hello::run_from_value(payload, app_state).await,
            _ => Err(miette::miette!("Unknown job type: {}", job.name)),
        }
    }
}
