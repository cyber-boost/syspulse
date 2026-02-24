use pyo3::exceptions::{PyConnectionError, PyRuntimeError, PyTimeoutError, PyValueError};
use pyo3::{create_exception, PyErr};

// Custom exception hierarchy
create_exception!(syspulse, SyspulseError, PyRuntimeError, "Base exception for all syspulse errors.");
create_exception!(syspulse, DaemonNotFoundError, PyValueError, "Raised when the requested daemon does not exist.");
create_exception!(syspulse, DaemonAlreadyExistsError, PyValueError, "Raised when a daemon with the given name already exists.");
create_exception!(syspulse, InvalidStateError, PyRuntimeError, "Raised on invalid daemon state transitions.");

pub fn to_py_err(err: syspulse_core::error::SyspulseError) -> PyErr {
    match err {
        syspulse_core::error::SyspulseError::DaemonNotFound(name) => {
            DaemonNotFoundError::new_err(format!("Daemon '{}' not found", name))
        }
        syspulse_core::error::SyspulseError::DaemonAlreadyExists(name) => {
            DaemonAlreadyExistsError::new_err(format!("Daemon '{}' already exists", name))
        }
        syspulse_core::error::SyspulseError::InvalidStateTransition { from, to } => {
            InvalidStateError::new_err(format!(
                "Invalid state transition from {:?} to {:?}",
                from, to
            ))
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
        other => SyspulseError::new_err(other.to_string()),
    }
}
