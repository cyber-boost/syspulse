use async_trait::async_trait;
use std::time::Duration;

use super::HealthChecker;
use crate::daemon::{HealthCheckSpec, HealthStatus};
use crate::error::{Result, SyspulseError};

pub struct HttpHealthChecker {
    spec: HealthCheckSpec,
    client: reqwest::Client,
}

impl HttpHealthChecker {
    pub fn new(spec: HealthCheckSpec) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(spec.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { spec, client }
    }
}

#[async_trait]
impl HealthChecker for HttpHealthChecker {
    async fn check(&self) -> Result<HealthStatus> {
        match self.client.get(&self.spec.target).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(HealthStatus::Healthy)
                } else {
                    tracing::debug!(
                        url = %self.spec.target,
                        status = %resp.status(),
                        "HTTP health check returned non-2xx"
                    );
                    Ok(HealthStatus::Unhealthy)
                }
            }
            Err(e) => {
                tracing::debug!(
                    url = %self.spec.target,
                    error = %e,
                    "HTTP health check failed"
                );
                if e.is_timeout() {
                    Err(SyspulseError::Timeout(Duration::from_secs(
                        self.spec.timeout_secs,
                    )))
                } else {
                    Ok(HealthStatus::Unhealthy)
                }
            }
        }
    }

    fn spec(&self) -> &HealthCheckSpec {
        &self.spec
    }
}
