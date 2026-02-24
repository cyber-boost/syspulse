use pyo3::prelude::*;

use syspulse_core::ipc::client::IpcClient;
use syspulse_core::ipc::protocol::{Request, Response};

use crate::daemon::Daemon;
use crate::errors::to_py_err;
use crate::instance::DaemonInstance;

#[pyclass]
pub struct SyspulseClient {
    client: IpcClient,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl SyspulseClient {
    #[new]
    #[pyo3(signature = (socket_path=None))]
    fn new(socket_path: Option<String>) -> PyResult<Self> {
        let path = socket_path
            .map(std::path::PathBuf::from)
            .unwrap_or_else(syspulse_core::paths::socket_path);
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self {
            client: IpcClient::new(path),
            runtime,
        })
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &self,
        _exc_type: Option<PyObject>,
        _exc_val: Option<PyObject>,
        _exc_tb: Option<PyObject>,
    ) -> bool {
        // No-op: tokio Runtime cleans up on Drop.
        false
    }

    #[pyo3(signature = (name, *, wait=None, timeout=None))]
    fn start(&self, name: &str, wait: Option<bool>, timeout: Option<u64>) -> PyResult<String> {
        let req = Request::Start {
            name: name.to_string(),
            wait: wait.unwrap_or(false),
            timeout_secs: timeout,
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        handle_ok_response(resp)
    }

    #[pyo3(signature = (name, *, force=None, timeout=None))]
    fn stop(&self, name: &str, force: Option<bool>, timeout: Option<u64>) -> PyResult<String> {
        let req = Request::Stop {
            name: name.to_string(),
            force: force.unwrap_or(false),
            timeout_secs: timeout,
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        handle_ok_response(resp)
    }

    #[pyo3(signature = (name, *, force=None, wait=None))]
    fn restart(&self, name: &str, force: Option<bool>, wait: Option<bool>) -> PyResult<String> {
        let req = Request::Restart {
            name: name.to_string(),
            force: force.unwrap_or(false),
            wait: wait.unwrap_or(false),
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        handle_ok_response(resp)
    }

    fn status(&self, name: &str) -> PyResult<DaemonInstance> {
        let req = Request::Status {
            name: Some(name.to_string()),
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        match resp {
            Response::Status { instance } => Ok(DaemonInstance::from_core(instance)),
            Response::Error { code, message } => Err(pyo3::exceptions::PyRuntimeError::new_err(
                format!("Error {}: {}", code, message),
            )),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Unexpected response",
            )),
        }
    }

    fn list(&self) -> PyResult<Vec<DaemonInstance>> {
        let req = Request::List;
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        match resp {
            Response::List { instances } => Ok(instances
                .into_iter()
                .map(DaemonInstance::from_core)
                .collect()),
            Response::Error { code, message } => Err(pyo3::exceptions::PyRuntimeError::new_err(
                format!("Error {}: {}", code, message),
            )),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Unexpected response",
            )),
        }
    }

    #[pyo3(signature = (name, *, lines=None, stderr=None))]
    fn logs(
        &self,
        name: &str,
        lines: Option<usize>,
        stderr: Option<bool>,
    ) -> PyResult<Vec<String>> {
        let req = Request::Logs {
            name: name.to_string(),
            lines: lines.unwrap_or(100),
            stderr: stderr.unwrap_or(false),
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        match resp {
            Response::Logs { lines } => Ok(lines),
            Response::Error { code, message } => Err(pyo3::exceptions::PyRuntimeError::new_err(
                format!("Error {}: {}", code, message),
            )),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Unexpected response",
            )),
        }
    }

    fn add(&self, daemon: &Daemon) -> PyResult<String> {
        let req = Request::Add {
            spec: daemon.inner.clone(),
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        handle_ok_response(resp)
    }

    #[pyo3(signature = (name, *, force=None))]
    fn remove(&self, name: &str, force: Option<bool>) -> PyResult<String> {
        let req = Request::Remove {
            name: name.to_string(),
            force: force.unwrap_or(false),
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        handle_ok_response(resp)
    }

    fn is_running(&self) -> bool {
        self.runtime.block_on(self.client.is_manager_running())
    }

    fn ping(&self) -> PyResult<bool> {
        let req = Request::Ping;
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        Ok(matches!(resp, Response::Pong))
    }

    fn shutdown(&self) -> PyResult<String> {
        let req = Request::Shutdown;
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        handle_ok_response(resp)
    }
}

fn handle_ok_response(resp: Response) -> PyResult<String> {
    match resp {
        Response::Ok { message } => Ok(message),
        Response::Error { code, message } => Err(pyo3::exceptions::PyRuntimeError::new_err(
            format!("Error {}: {}", code, message),
        )),
        _ => Ok("OK".to_string()),
    }
}
