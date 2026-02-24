use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum DaemonStatus {
    Stopped = 0,
    Starting = 1,
    Running = 2,
    Stopping = 3,
    Failed = 4,
    Scheduled = 5,
}

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum HealthStatus {
    Unknown = 0,
    Healthy = 1,
    Unhealthy = 2,
    NotConfigured = 3,
}

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum RestartPolicyType {
    Always = 0,
    OnFailure = 1,
    Never = 2,
}

impl From<syspulse_core::lifecycle::LifecycleState> for DaemonStatus {
    fn from(state: syspulse_core::lifecycle::LifecycleState) -> Self {
        match state {
            syspulse_core::lifecycle::LifecycleState::Stopped => DaemonStatus::Stopped,
            syspulse_core::lifecycle::LifecycleState::Starting => DaemonStatus::Starting,
            syspulse_core::lifecycle::LifecycleState::Running => DaemonStatus::Running,
            syspulse_core::lifecycle::LifecycleState::Stopping => DaemonStatus::Stopping,
            syspulse_core::lifecycle::LifecycleState::Failed => DaemonStatus::Failed,
            syspulse_core::lifecycle::LifecycleState::Scheduled => DaemonStatus::Scheduled,
        }
    }
}

impl From<syspulse_core::daemon::HealthStatus> for HealthStatus {
    fn from(status: syspulse_core::daemon::HealthStatus) -> Self {
        match status {
            syspulse_core::daemon::HealthStatus::Unknown => HealthStatus::Unknown,
            syspulse_core::daemon::HealthStatus::Healthy => HealthStatus::Healthy,
            syspulse_core::daemon::HealthStatus::Unhealthy => HealthStatus::Unhealthy,
            syspulse_core::daemon::HealthStatus::NotConfigured => HealthStatus::NotConfigured,
        }
    }
}

impl From<&syspulse_core::restart::RestartPolicy> for RestartPolicyType {
    fn from(policy: &syspulse_core::restart::RestartPolicy) -> Self {
        match policy {
            syspulse_core::restart::RestartPolicy::Always { .. } => RestartPolicyType::Always,
            syspulse_core::restart::RestartPolicy::OnFailure { .. } => RestartPolicyType::OnFailure,
            syspulse_core::restart::RestartPolicy::Never => RestartPolicyType::Never,
        }
    }
}
