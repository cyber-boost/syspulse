use pyo3::prelude::*;

/// Type of health check to perform.
#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum HealthCheckType {
    Http = 0,
    Tcp = 1,
    Command = 2,
}

impl From<&syspulse_core::daemon::HealthCheckType> for HealthCheckType {
    fn from(t: &syspulse_core::daemon::HealthCheckType) -> Self {
        match t {
            syspulse_core::daemon::HealthCheckType::Http => HealthCheckType::Http,
            syspulse_core::daemon::HealthCheckType::Tcp => HealthCheckType::Tcp,
            syspulse_core::daemon::HealthCheckType::Command => HealthCheckType::Command,
        }
    }
}

impl From<&HealthCheckType> for syspulse_core::daemon::HealthCheckType {
    fn from(t: &HealthCheckType) -> Self {
        match t {
            HealthCheckType::Http => syspulse_core::daemon::HealthCheckType::Http,
            HealthCheckType::Tcp => syspulse_core::daemon::HealthCheckType::Tcp,
            HealthCheckType::Command => syspulse_core::daemon::HealthCheckType::Command,
        }
    }
}

/// Health check configuration for a daemon.
#[pyclass]
#[derive(Clone)]
pub struct HealthCheck {
    pub(crate) inner: syspulse_core::daemon::HealthCheckSpec,
}

#[pymethods]
impl HealthCheck {
    #[new]
    #[pyo3(signature = (check_type, target, *, interval=30, timeout=5, retries=3, start_period=0))]
    fn new(
        check_type: &HealthCheckType,
        target: String,
        interval: u64,
        timeout: u64,
        retries: u32,
        start_period: u64,
    ) -> Self {
        HealthCheck {
            inner: syspulse_core::daemon::HealthCheckSpec {
                check_type: syspulse_core::daemon::HealthCheckType::from(check_type),
                target,
                interval_secs: interval,
                timeout_secs: timeout,
                retries,
                start_period_secs: start_period,
            },
        }
    }

    #[getter]
    fn check_type(&self) -> HealthCheckType {
        HealthCheckType::from(&self.inner.check_type)
    }

    #[getter]
    fn target(&self) -> &str {
        &self.inner.target
    }

    #[getter]
    fn interval(&self) -> u64 {
        self.inner.interval_secs
    }

    #[getter]
    fn timeout(&self) -> u64 {
        self.inner.timeout_secs
    }

    #[getter]
    fn retries(&self) -> u32 {
        self.inner.retries
    }

    #[getter]
    fn start_period(&self) -> u64 {
        self.inner.start_period_secs
    }

    fn __repr__(&self) -> String {
        format!(
            "HealthCheck(check_type={:?}, target='{}', interval={})",
            self.inner.check_type, self.inner.target, self.inner.interval_secs
        )
    }
}
