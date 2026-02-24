use std::ffi::CStr;

use pyo3::prelude::*;

use crate::daemon::Daemon;
use crate::errors::to_py_err;

/// Load daemons from a .sys configuration file.
#[pyfunction]
pub fn from_sys(path: &str) -> PyResult<Vec<Daemon>> {
    let specs =
        syspulse_core::config::parse_config_file(std::path::Path::new(path)).map_err(to_py_err)?;
    Ok(specs.into_iter().map(|s| Daemon { inner: s }).collect())
}

const DEPRECATION_MSG: &CStr = c"from_toml() is deprecated, use from_sys() instead";

/// Load daemons from a config file.
///
/// .. deprecated::
///     Use :func:`from_sys` instead.
#[pyfunction]
pub fn from_toml(py: Python<'_>, path: &str) -> PyResult<Vec<Daemon>> {
    PyErr::warn(
        py,
        &py.get_type::<pyo3::exceptions::PyDeprecationWarning>(),
        DEPRECATION_MSG,
        1,
    )?;
    from_sys(path)
}
