use chrono::Duration;
use cja::{app_state::AppState as _, jobs::Job};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use tokio::time::Instant;
use uuid::Uuid;

use crate::app_state::AppState;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateCheckin {
    pub page_id: Uuid,
}

#[async_trait::async_trait]
impl Job<AppState> for CreateCheckin {
    const NAME: &'static str = "CreateCheckin";

    async fn run(&self, app_state: AppState) -> miette::Result<()> {
        let page = sqlx::query!(
            r#"
        SELECT Pages.path, Sites.domain
        FROM Pages
        JOIN Sites ON Sites.site_id = Pages.site_id
        WHERE page_id = $1
      "#,
            self.page_id
        )
        .fetch_one(app_state.db())
        .await
        .into_diagnostic()?;

        let domain = page.domain;
        let path = page.path;

        let url = format!("https://{domain}{path}");
        let now = Instant::now();
        let resp = reqwest::get(&url).await;

        let (status, outcome) = match resp {
            Err(_) => (None, "error"),
            Ok(resp) => (Some(resp.status()), {
                if resp.status().is_success() {
                    "success"
                } else {
                    "failure"
                }
            }),
        };
        let duration = now.elapsed();
        let duration: i64 = duration.as_nanos().try_into().unwrap();

        let status: Option<i32> = status.map(|s| s.as_u16().into());

        sqlx::query!(
            r#"
        INSERT INTO Checkins (page_id, status_code, outcome, duration_nanos)
        VALUES ($1, $2, $3, $4)
      "#,
            self.page_id,
            status,
            outcome,
            duration
        )
        .execute(app_state.db())
        .await
        .into_diagnostic()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BulkEnqueueCheckins;

#[async_trait::async_trait]
impl Job<AppState> for BulkEnqueueCheckins {
    const NAME: &'static str = "BulkEnqueueCheckins";

    async fn run(&self, app_state: AppState) -> miette::Result<()> {
        let pages = sqlx::query!(
            r#"
        SELECT page_id
        FROM Pages
      "#
        )
        .fetch_all(app_state.db())
        .await
        .into_diagnostic()?;

        for page in pages {
            let job = CreateCheckin {
                page_id: page.page_id,
            };

            job.enqueue(app_state.clone(), "Bulk Checkin Enqueue".to_string())
                .await?;
        }

        Ok(())
    }
}
