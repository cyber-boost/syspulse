use pyo3::prelude::*;

/// Log rotation configuration for a daemon.
#[pyclass]
#[derive(Clone)]
pub struct LogConfig {
    pub(crate) inner: syspulse_core::daemon::LogConfig,
}

#[pymethods]
impl LogConfig {
    #[new]
    #[pyo3(signature = (*, max_size_bytes=52428800, retain_count=5, compress_rotated=false))]
    fn new(max_size_bytes: u64, retain_count: u32, compress_rotated: bool) -> Self {
        LogConfig {
            inner: syspulse_core::daemon::LogConfig {
                max_size_bytes,
                retain_count,
                compress_rotated,
            },
        }
    }

    #[getter]
    fn max_size_bytes(&self) -> u64 {
        self.inner.max_size_bytes
    }

    #[getter]
    fn retain_count(&self) -> u32 {
        self.inner.retain_count
    }

    #[getter]
    fn compress_rotated(&self) -> bool {
        self.inner.compress_rotated
    }

    fn __repr__(&self) -> String {
        format!(
            "LogConfig(max_size_bytes={}, retain_count={}, compress_rotated={})",
            self.inner.max_size_bytes, self.inner.retain_count, self.inner.compress_rotated
        )
    }
}
