use pyo3::prelude::*;

mod config;
mod daemon;
mod errors;
mod health;
mod instance;
mod log_config;
mod manager;
mod paths;
mod resources;
mod types;

#[pymodule]
fn _syspulse(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Core classes
    m.add_class::<daemon::Daemon>()?;
    m.add_class::<instance::DaemonInstance>()?;
    m.add_class::<manager::SyspulseClient>()?;

    // Config classes
    m.add_class::<health::HealthCheck>()?;
    m.add_class::<resources::ResourceLimits>()?;
    m.add_class::<log_config::LogConfig>()?;

    // Enums
    m.add_class::<types::DaemonStatus>()?;
    m.add_class::<types::HealthStatus>()?;
    m.add_class::<types::RestartPolicyType>()?;
    m.add_class::<health::HealthCheckType>()?;

    // Exceptions
    m.add(
        "SyspulseError",
        m.py().get_type::<errors::SyspulseError>(),
    )?;
    m.add(
        "DaemonNotFoundError",
        m.py().get_type::<errors::DaemonNotFoundError>(),
    )?;
    m.add(
        "DaemonAlreadyExistsError",
        m.py().get_type::<errors::DaemonAlreadyExistsError>(),
    )?;
    m.add(
        "InvalidStateError",
        m.py().get_type::<errors::InvalidStateError>(),
    )?;

    // Functions
    m.add_function(wrap_pyfunction!(config::from_sys, m)?)?;
    m.add_function(wrap_pyfunction!(config::from_toml, m)?)?;
    m.add_function(wrap_pyfunction!(paths::data_dir, m)?)?;
    m.add_function(wrap_pyfunction!(paths::socket_path, m)?)?;

    Ok(())
}
