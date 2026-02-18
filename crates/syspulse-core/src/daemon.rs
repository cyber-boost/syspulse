use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::lifecycle::LifecycleState;
use crate::resources::ResourceLimits;
use crate::restart::RestartPolicy;

fn default_interval() -> u64 {
    30
}

fn default_timeout() -> u64 {
    5
}

fn default_retries() -> u32 {
    3
}

fn default_max_size() -> u64 {
    50 * 1024 * 1024 // 50 MB
}

fn default_retain() -> u32 {
    5
}

fn default_stop_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    #[serde(rename = "type")]
    pub check_type: HealthCheckType,
    pub target: String,
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_retries")]
    pub retries: u32,
    #[serde(default)]
    pub start_period_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthCheckType {
    Http,
    Tcp,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_max_size")]
    pub max_size_bytes: u64,
    #[serde(default = "default_retain")]
    pub retain_count: u32,
    #[serde(default)]
    pub compress_rotated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSpec {
    pub name: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub health_check: Option<HealthCheckSpec>,
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    #[serde(default)]
    pub resource_limits: Option<ResourceLimits>,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_stop_timeout")]
    pub stop_timeout_secs: u64,
    #[serde(default)]
    pub log_config: Option<LogConfig>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInstance {
    pub id: String,
    pub spec_name: String,
    pub state: LifecycleState,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub restart_count: u32,
    pub health_status: HealthStatus,
    pub stdout_log: Option<PathBuf>,
    pub stderr_log: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Unhealthy,
    NotConfigured,
}

impl DaemonInstance {
    pub fn new(spec_name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            spec_name: spec_name.to_string(),
            state: LifecycleState::Stopped,
            pid: None,
            started_at: None,
            stopped_at: None,
            exit_code: None,
            restart_count: 0,
            health_status: HealthStatus::Unknown,
            stdout_log: None,
            stderr_log: None,
        }
    }
}
