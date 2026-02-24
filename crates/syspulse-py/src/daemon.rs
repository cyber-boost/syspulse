use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::health::HealthCheck;
use crate::log_config::LogConfig;
use crate::resources::ResourceLimits;
use crate::types::RestartPolicyType;

#[pyclass]
#[derive(Clone)]
pub struct Daemon {
    pub(crate) inner: syspulse_core::daemon::DaemonSpec,
}

#[pymethods]
impl Daemon {
    #[new]
    #[pyo3(signature = (name, command, *, working_dir=None, env=None, schedule=None, tags=None, stop_timeout=30, description=None, user=None, health_check=None, resource_limits=None, log_config=None))]
    fn new(
        name: String,
        command: Vec<String>,
        working_dir: Option<String>,
        env: Option<HashMap<String, String>>,
        schedule: Option<String>,
        tags: Option<Vec<String>>,
        stop_timeout: u64,
        description: Option<String>,
        user: Option<String>,
        health_check: Option<HealthCheck>,
        resource_limits: Option<ResourceLimits>,
        log_config: Option<LogConfig>,
    ) -> Self {
        Daemon {
            inner: syspulse_core::daemon::DaemonSpec {
                name,
                command,
                working_dir: working_dir.map(PathBuf::from),
                env: env.unwrap_or_default(),
                health_check: health_check.map(|h| h.inner),
                restart_policy: Default::default(),
                resource_limits: resource_limits.map(|r| r.inner),
                schedule,
                tags: tags.unwrap_or_default(),
                stop_timeout_secs: stop_timeout,
                log_config: log_config.map(|l| l.inner),
                description,
                user,
            },
        }
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn command(&self) -> Vec<String> {
        self.inner.command.clone()
    }

    #[getter]
    fn working_dir(&self) -> Option<String> {
        self.inner
            .working_dir
            .as_ref()
            .map(|p| p.display().to_string())
    }

    #[getter]
    fn env(&self) -> HashMap<String, String> {
        self.inner.env.clone()
    }

    #[getter]
    fn schedule(&self) -> Option<String> {
        self.inner.schedule.clone()
    }

    #[getter]
    fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    #[getter]
    fn stop_timeout(&self) -> u64 {
        self.inner.stop_timeout_secs
    }

    #[getter]
    fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    #[getter]
    fn user(&self) -> Option<String> {
        self.inner.user.clone()
    }

    #[getter]
    fn restart_policy(&self) -> RestartPolicyType {
        RestartPolicyType::from(&self.inner.restart_policy)
    }

    #[getter]
    fn health_check(&self) -> Option<HealthCheck> {
        self.inner
            .health_check
            .as_ref()
            .map(|h| HealthCheck { inner: h.clone() })
    }

    #[getter]
    fn resource_limits(&self) -> Option<ResourceLimits> {
        self.inner
            .resource_limits
            .as_ref()
            .map(|r| ResourceLimits { inner: r.clone() })
    }

    #[getter]
    fn log_config(&self) -> Option<LogConfig> {
        self.inner
            .log_config
            .as_ref()
            .map(|l| LogConfig { inner: l.clone() })
    }

    fn __repr__(&self) -> String {
        format!(
            "Daemon(name='{}', command={:?})",
            self.inner.name, self.inner.command
        )
    }

    /// Builder: set a health check (backward-compatible string-based API).
    #[pyo3(signature = (check_type, target, *, interval=None, timeout=None, retries=None, start_period=None))]
    fn with_health_check(
        &self,
        check_type: &str,
        target: &str,
        interval: Option<u64>,
        timeout: Option<u64>,
        retries: Option<u32>,
        start_period: Option<u64>,
    ) -> Self {
        let mut d = self.clone();
        d.inner.health_check = Some(syspulse_core::daemon::HealthCheckSpec {
            check_type: match check_type {
                "http" => syspulse_core::daemon::HealthCheckType::Http,
                "tcp" => syspulse_core::daemon::HealthCheckType::Tcp,
                _ => syspulse_core::daemon::HealthCheckType::Command,
            },
            target: target.to_string(),
            interval_secs: interval.unwrap_or(30),
            timeout_secs: timeout.unwrap_or(5),
            retries: retries.unwrap_or(3),
            start_period_secs: start_period.unwrap_or(0),
        });
        d
    }

    /// Builder: set a restart policy.
    #[pyo3(signature = (policy, *, max_retries=None, backoff_base=None, backoff_max=None))]
    fn with_restart_policy(
        &self,
        policy: &str,
        max_retries: Option<u32>,
        backoff_base: Option<f64>,
        backoff_max: Option<f64>,
    ) -> Self {
        let mut d = self.clone();
        d.inner.restart_policy = match policy {
            "always" => syspulse_core::restart::RestartPolicy::Always {
                max_retries,
                backoff_base_secs: backoff_base.unwrap_or(1.0),
                backoff_max_secs: backoff_max.unwrap_or(300.0),
            },
            "on_failure" => syspulse_core::restart::RestartPolicy::OnFailure {
                max_retries,
                backoff_base_secs: backoff_base.unwrap_or(1.0),
                backoff_max_secs: backoff_max.unwrap_or(300.0),
            },
            _ => syspulse_core::restart::RestartPolicy::Never,
        };
        d
    }

    /// Builder: set resource limits.
    fn with_resource_limits(&self, limits: ResourceLimits) -> Self {
        let mut d = self.clone();
        d.inner.resource_limits = Some(limits.inner);
        d
    }

    /// Builder: set log configuration.
    fn with_log_config(&self, config: LogConfig) -> Self {
        let mut d = self.clone();
        d.inner.log_config = Some(config.inner);
        d
    }
}
