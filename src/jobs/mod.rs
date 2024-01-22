pub(crate) mod create_checkin;
pub(crate) mod hello;

use cja::{
    impl_job_registry,
    jobs::{registry::JobRegistry, worker::JobFromDB, Job},
};

use crate::AppState;

impl_job_registry!(
    AppState,
    hello::Hello,
    create_checkin::CreateCheckin,
    create_checkin::BulkEnqueueCheckins
);
