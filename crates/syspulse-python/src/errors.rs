use pyo3::exceptions::{PyConnectionError, PyRuntimeError, PyTimeoutError, PyValueError};
use pyo3::PyErr;

pub fn to_py_err(err: syspulse_core::error::SyspulseError) -> PyErr {
    match err {
        syspulse_core::error::SyspulseError::DaemonNotFound(name) => {
            PyValueError::new_err(format!("Daemon '{}' not found", name))
        }
        syspulse_core::error::SyspulseError::DaemonAlreadyExists(name) => {
            PyValueError::new_err(format!("Daemon '{}' already exists", name))
        }
        syspulse_core::error::SyspulseError::Ipc(msg) => {
            PyConnectionError::new_err(format!("IPC error: {}", msg))
        }
        syspulse_core::error::SyspulseError::Timeout(d) => {
            PyTimeoutError::new_err(format!("Timeout after {:?}", d))
        }
        syspulse_core::error::SyspulseError::Config(msg) => {
            PyValueError::new_err(format!("Config error: {}", msg))
        }
        other => PyRuntimeError::new_err(other.to_string()),
    }
}
