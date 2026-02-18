use async_trait::async_trait;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::HealthChecker;
use crate::daemon::{HealthCheckSpec, HealthStatus};
use crate::error::{Result, SyspulseError};

pub struct TcpHealthChecker {
    spec: HealthCheckSpec,
}

impl TcpHealthChecker {
    pub fn new(spec: HealthCheckSpec) -> Self {
        Self { spec }
    }
}

#[async_trait]
impl HealthChecker for TcpHealthChecker {
    async fn check(&self) -> Result<HealthStatus> {
        let dur = Duration::from_secs(self.spec.timeout_secs);

        match timeout(dur, TcpStream::connect(&self.spec.target)).await {
            Ok(Ok(_stream)) => Ok(HealthStatus::Healthy),
            Ok(Err(e)) => {
                tracing::debug!(
                    target = %self.spec.target,
                    error = %e,
                    "TCP health check connection failed"
                );
                Ok(HealthStatus::Unhealthy)
            }
            Err(_) => {
                tracing::debug!(
                    target = %self.spec.target,
                    "TCP health check timed out"
                );
                Err(SyspulseError::Timeout(dur))
            }
        }
    }

    fn spec(&self) -> &HealthCheckSpec {
        &self.spec
    }
}
