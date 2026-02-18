use pyo3::prelude::*;
use pyo3::types::PyDict;

use syspulse_core::ipc::client::IpcClient;
use syspulse_core::ipc::protocol::{Request, Response};

use crate::daemon::Daemon;
use crate::errors::to_py_err;

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

    fn status(&self, name: &str) -> PyResult<PyObject> {
        let req = Request::Status {
            name: Some(name.to_string()),
        };
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        Python::with_gil(|py| match resp {
            Response::Status { instance } => {
                let dict = instance_to_dict(py, &instance)?;
                Ok(dict.into())
            }
            Response::Error { code, message } => Err(
                pyo3::exceptions::PyRuntimeError::new_err(format!("Error {}: {}", code, message)),
            ),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Unexpected response",
            )),
        })
    }

    fn list(&self) -> PyResult<Vec<PyObject>> {
        let req = Request::List;
        let resp = self
            .runtime
            .block_on(self.client.send(req))
            .map_err(to_py_err)?;
        Python::with_gil(|py| match resp {
            Response::List { instances } => {
                let mut results = Vec::with_capacity(instances.len());
                for instance in &instances {
                    let dict = instance_to_dict(py, instance)?;
                    results.push(dict.into());
                }
                Ok(results)
            }
            Response::Error { code, message } => Err(
                pyo3::exceptions::PyRuntimeError::new_err(format!("Error {}: {}", code, message)),
            ),
            _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Unexpected response",
            )),
        })
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
            Response::Error { code, message } => Err(
                pyo3::exceptions::PyRuntimeError::new_err(format!("Error {}: {}", code, message)),
            ),
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

fn instance_to_dict<'py>(
    py: Python<'py>,
    instance: &syspulse_core::daemon::DaemonInstance,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("id", &instance.id)?;
    dict.set_item("name", &instance.spec_name)?;
    dict.set_item("state", instance.state.to_string())?;
    dict.set_item("pid", instance.pid)?;
    dict.set_item("exit_code", instance.exit_code)?;
    dict.set_item("restart_count", instance.restart_count)?;
    dict.set_item("health", format!("{:?}", instance.health_status))?;
    dict.set_item(
        "started_at",
        instance.started_at.map(|t| t.to_rfc3339()),
    )?;
    dict.set_item(
        "stopped_at",
        instance.stopped_at.map(|t| t.to_rfc3339()),
    )?;
    dict.set_item(
        "stdout_log",
        instance.stdout_log.as_ref().map(|p| p.display().to_string()),
    )?;
    dict.set_item(
        "stderr_log",
        instance.stderr_log.as_ref().map(|p| p.display().to_string()),
    )?;
    Ok(dict)
}
