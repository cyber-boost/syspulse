use pyo3::prelude::*;

use crate::types::{DaemonStatus, HealthStatus};

/// Immutable snapshot of a running (or stopped) daemon instance.
#[pyclass(frozen)]
#[derive(Clone)]
pub struct DaemonInstance {
    inner: syspulse_core::daemon::DaemonInstance,
}

impl DaemonInstance {
    pub fn from_core(instance: syspulse_core::daemon::DaemonInstance) -> Self {
        Self { inner: instance }
    }
}

#[pymethods]
impl DaemonInstance {
    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.spec_name
    }

    #[getter]
    fn state(&self) -> DaemonStatus {
        DaemonStatus::from(self.inner.state.clone())
    }

    #[getter]
    fn pid(&self) -> Option<u32> {
        self.inner.pid
    }

    #[getter]
    fn started_at(&self) -> Option<String> {
        self.inner.started_at.map(|t| t.to_rfc3339())
    }

    #[getter]
    fn stopped_at(&self) -> Option<String> {
        self.inner.stopped_at.map(|t| t.to_rfc3339())
    }

    #[getter]
    fn exit_code(&self) -> Option<i32> {
        self.inner.exit_code
    }

    #[getter]
    fn restart_count(&self) -> u32 {
        self.inner.restart_count
    }

    #[getter]
    fn health(&self) -> HealthStatus {
        HealthStatus::from(self.inner.health_status.clone())
    }

    #[getter]
    fn stdout_log(&self) -> Option<String> {
        self.inner
            .stdout_log
            .as_ref()
            .map(|p| p.display().to_string())
    }

    #[getter]
    fn stderr_log(&self) -> Option<String> {
        self.inner
            .stderr_log
            .as_ref()
            .map(|p| p.display().to_string())
    }

    fn __repr__(&self) -> String {
        format!(
            "DaemonInstance(name='{}', state={:?}, pid={:?})",
            self.inner.spec_name, self.inner.state, self.inner.pid
        )
    }
}
