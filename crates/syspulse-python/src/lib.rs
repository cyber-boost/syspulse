use pyo3::prelude::*;

mod config;
mod daemon;
mod errors;
mod manager;
mod types;

#[pymodule]
fn _syspulse(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<daemon::Daemon>()?;
    m.add_class::<manager::SyspulseClient>()?;
    m.add_class::<types::DaemonStatus>()?;
    m.add_class::<types::HealthStatus>()?;
    m.add_class::<types::RestartPolicyType>()?;
    m.add_function(wrap_pyfunction!(config::from_toml, m)?)?;
    Ok(())
}
