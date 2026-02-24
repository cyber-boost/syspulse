use pyo3::prelude::*;

/// Resource limits for a daemon process.
#[pyclass]
#[derive(Clone)]
pub struct ResourceLimits {
    pub(crate) inner: syspulse_core::resources::ResourceLimits,
}

#[pymethods]
impl ResourceLimits {
    #[new]
    #[pyo3(signature = (*, max_memory_bytes=None, max_cpu_percent=None, max_open_files=None))]
    fn new(
        max_memory_bytes: Option<u64>,
        max_cpu_percent: Option<f64>,
        max_open_files: Option<u64>,
    ) -> Self {
        ResourceLimits {
            inner: syspulse_core::resources::ResourceLimits {
                max_memory_bytes,
                max_cpu_percent,
                max_open_files,
            },
        }
    }

    #[getter]
    fn max_memory_bytes(&self) -> Option<u64> {
        self.inner.max_memory_bytes
    }

    #[getter]
    fn max_cpu_percent(&self) -> Option<f64> {
        self.inner.max_cpu_percent
    }

    #[getter]
    fn max_open_files(&self) -> Option<u64> {
        self.inner.max_open_files
    }

    fn __repr__(&self) -> String {
        format!(
            "ResourceLimits(max_memory_bytes={:?}, max_cpu_percent={:?}, max_open_files={:?})",
            self.inner.max_memory_bytes, self.inner.max_cpu_percent, self.inner.max_open_files
        )
    }
}
