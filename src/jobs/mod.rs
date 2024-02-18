pub(crate) mod create_checkin;
pub(crate) mod hello;

pub struct Jobs;

#[async_trait::async_trait]
impl cja::jobs::registry::JobRegistry<crate::AppState> for Jobs {
    async fn run_job(
        &self,
        job: &cja::jobs::worker::JobFromDB,
        app_state: crate::AppState,
    ) -> miette::Result<()> {
        use cja::jobs::Job as _;
        let payload = job.payload.clone();
        match job.name.as_str() {
            <hello::Hello>::NAME => <hello::Hello>::run_from_value(payload, app_state).await,
            <create_checkin::CreateCheckin>::NAME => {
                <create_checkin::CreateCheckin>::run_from_value(payload, app_state).await
            }
            <create_checkin::BulkEnqueueCheckins>::NAME => {
                <create_checkin::BulkEnqueueCheckins>::run_from_value(payload, app_state).await
            }
            _ => Err(miette::miette!("Unknown job type: {}", job.name)),
        }
    }
}
