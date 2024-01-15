use std::time::Duration;

use cja::{
    cron::{CronRegistry, Worker},
    jobs::Job as _,
};

use crate::{app_state::AppState, jobs::hello::Hello};

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register(
        "HelloWorld",
        Duration::from_secs(60),
        |app_state: AppState, context| Hello.enqueue(app_state.clone(), context),
    );

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> miette::Result<()> {
    Worker::new(app_state, cron_registry()).run().await?;

    Ok(())
}
