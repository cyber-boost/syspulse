use async_trait::async_trait;
use std::path::PathBuf;

use crate::daemon::DaemonSpec;
use crate::error::Result;

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub alive: bool,
}

#[derive(Debug, Default)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub cpu_percent: f64,
}

#[async_trait]
pub trait ProcessDriver: Send + Sync {
    async fn spawn(
        &self,
        spec: &DaemonSpec,
        stdout_path: &PathBuf,
        stderr_path: &PathBuf,
    ) -> Result<ProcessInfo>;

    async fn stop(&self, pid: u32, timeout_secs: u64) -> Result<()>;
    async fn kill(&self, pid: u32) -> Result<()>;
    async fn is_alive(&self, pid: u32) -> bool;
    async fn wait(&self, pid: u32) -> Result<Option<i32>>;
    async fn resource_usage(&self, pid: u32) -> Result<ResourceUsage>;
}

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub fn create_driver() -> Box<dyn ProcessDriver> {
    #[cfg(unix)]
    {
        Box::new(unix::UnixProcessDriver::new())
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsProcessDriver::new())
    }
}
