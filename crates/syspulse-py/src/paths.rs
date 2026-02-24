use pyo3::prelude::*;

/// Return the platform-specific data directory path.
#[pyfunction]
pub fn data_dir() -> String {
    syspulse_core::paths::data_dir().display().to_string()
}

/// Return the platform-specific socket/pipe path.
#[pyfunction]
pub fn socket_path() -> String {
    syspulse_core::paths::socket_path().display().to_string()
}
