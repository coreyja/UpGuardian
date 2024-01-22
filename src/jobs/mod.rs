pub(crate) mod create_checkin;
pub(crate) mod hello;

cja::impl_job_registry!(
    crate::AppState,
    hello::Hello,
    create_checkin::CreateCheckin,
    create_checkin::BulkEnqueueCheckins
);
