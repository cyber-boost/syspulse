use pyo3::prelude::*;

use crate::daemon::Daemon;
use crate::errors::to_py_err;

#[pyfunction]
pub fn from_toml(path: &str) -> PyResult<Vec<Daemon>> {
    let specs =
        syspulse_core::config::parse_config_file(std::path::Path::new(path)).map_err(to_py_err)?;
    Ok(specs.into_iter().map(|s| Daemon { inner: s }).collect())
}
