use async_trait::async_trait;

use crate::daemon::{HealthCheckSpec, HealthCheckType, HealthStatus};
use crate::error::Result;

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check(&self) -> Result<HealthStatus>;
    fn spec(&self) -> &HealthCheckSpec;
}

pub mod command;
pub mod http;
pub mod tcp;

pub fn create_checker(spec: HealthCheckSpec) -> Box<dyn HealthChecker> {
    match spec.check_type {
        HealthCheckType::Http => Box::new(http::HttpHealthChecker::new(spec)),
        HealthCheckType::Tcp => Box::new(tcp::TcpHealthChecker::new(spec)),
        HealthCheckType::Command => Box::new(command::CommandHealthChecker::new(spec)),
    }
}
