use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;

use super::HealthChecker;
use crate::daemon::{HealthCheckSpec, HealthStatus};
use crate::error::{Result, SyspulseError};

pub struct CommandHealthChecker {
    spec: HealthCheckSpec,
}

impl CommandHealthChecker {
    pub fn new(spec: HealthCheckSpec) -> Self {
        Self { spec }
    }
}

#[async_trait]
impl HealthChecker for CommandHealthChecker {
    async fn check(&self) -> Result<HealthStatus> {
        let dur = Duration::from_secs(self.spec.timeout_secs);

        #[cfg(unix)]
        let mut cmd = {
            let mut c = tokio::process::Command::new("sh");
            c.arg("-c").arg(&self.spec.target);
            c
        };

        #[cfg(windows)]
        let mut cmd = {
            let mut c = tokio::process::Command::new("cmd");
            c.arg("/C").arg(&self.spec.target);
            c
        };

        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        match timeout(dur, cmd.status()).await {
            Ok(Ok(status)) => {
                if status.success() {
                    Ok(HealthStatus::Healthy)
                } else {
                    tracing::debug!(
                        command = %self.spec.target,
                        exit_code = ?status.code(),
                        "Command health check returned non-zero"
                    );
                    Ok(HealthStatus::Unhealthy)
                }
            }
            Ok(Err(e)) => {
                tracing::debug!(
                    command = %self.spec.target,
                    error = %e,
                    "Command health check failed to execute"
                );
                Ok(HealthStatus::Unhealthy)
            }
            Err(_) => {
                tracing::debug!(
                    command = %self.spec.target,
                    "Command health check timed out"
                );
                Err(SyspulseError::Timeout(dur))
            }
        }
    }

    fn spec(&self) -> &HealthCheckSpec {
        &self.spec
    }
}
