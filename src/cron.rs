use std::time::Duration;

use cja::{
    cron::{CronRegistry, Worker},
    jobs::Job as _,
};

use crate::{
    app_state::AppState,
    jobs::{create_checkin::BulkEnqueueCheckins, hello::Hello},
};

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register_job(Hello, Duration::from_secs(60));
    registry.register_job(BulkEnqueueCheckins, Duration::from_secs(60));

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> miette::Result<()> {
    Worker::new(app_state, cron_registry()).run().await?;

    Ok(())
}
