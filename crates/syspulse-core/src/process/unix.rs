use async_trait::async_trait;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tokio::time::{sleep, Duration};

use super::{ProcessDriver, ProcessInfo, ResourceUsage};
use crate::daemon::DaemonSpec;
use crate::error::{Result, SyspulseError};

pub struct UnixProcessDriver;

impl UnixProcessDriver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProcessDriver for UnixProcessDriver {
    async fn spawn(
        &self,
        spec: &DaemonSpec,
        stdout_path: &PathBuf,
        stderr_path: &PathBuf,
    ) -> Result<ProcessInfo> {
        let stdout_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(stdout_path)?;
        let stderr_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(stderr_path)?;

        let program = &spec.command[0];
        let args = &spec.command[1..];

        let mut cmd = tokio::process::Command::new(program);
        cmd.args(args)
            .stdout(std::process::Stdio::from(stdout_file))
            .stderr(std::process::Stdio::from(stderr_file))
            .stdin(std::process::Stdio::null());

        if let Some(ref dir) = spec.working_dir {
            cmd.current_dir(dir);
        }

        for (key, val) in &spec.env {
            cmd.env(key, val);
        }

        let resource_limits = spec.resource_limits.clone();
        unsafe {
            cmd.pre_exec(move || {
                // Create new session so the daemon runs independently
                libc::setsid();

                // Apply resource limits if configured
                if let Some(ref limits) = resource_limits {
                    if let Some(max_mem) = limits.max_memory_bytes {
                        rlimit::setrlimit(rlimit::Resource::AS, max_mem, max_mem)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    }
                    if let Some(max_files) = limits.max_open_files {
                        rlimit::setrlimit(rlimit::Resource::NOFILE, max_files, max_files)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    }
                }
                Ok(())
            });
        }

        // Don't kill child when the handle is dropped
        cmd.kill_on_drop(false);

        let child = cmd
            .spawn()
            .map_err(|e| SyspulseError::Process(format!("Failed to spawn process: {}", e)))?;

        let pid = child
            .id()
            .ok_or_else(|| SyspulseError::Process("Failed to get child PID".into()))?;

        // Detach: drop the child handle so we don't wait on it
        std::mem::forget(child);

        Ok(ProcessInfo { pid, alive: true })
    }

    async fn stop(&self, pid: u32, timeout_secs: u64) -> Result<()> {
        let pgid = Pid::from_raw(-(pid as i32));

        // Send SIGTERM to the process group
        if signal::kill(pgid, Signal::SIGTERM).is_err() {
            return Ok(());
        }

        // Wait up to timeout for process to exit
        let deadline = Duration::from_secs(timeout_secs);
        let interval = Duration::from_millis(100);
        let mut elapsed = Duration::ZERO;

        while elapsed < deadline {
            if !self.is_alive(pid).await {
                return Ok(());
            }
            sleep(interval).await;
            elapsed += interval;
        }

        // Timed out: send SIGKILL
        tracing::warn!(pid, "Process did not exit after SIGTERM, sending SIGKILL");
        self.kill(pid).await
    }

    async fn kill(&self, pid: u32) -> Result<()> {
        let pgid = Pid::from_raw(-(pid as i32));
        signal::kill(pgid, Signal::SIGKILL).map_err(|e| {
            SyspulseError::Process(format!("Failed to kill process {}: {}", pid, e))
        })?;
        Ok(())
    }

    async fn is_alive(&self, pid: u32) -> bool {
        let pid = Pid::from_raw(pid as i32);
        // Sending signal 0 checks if the process exists
        signal::kill(pid, None).is_ok()
    }

    async fn wait(&self, pid: u32) -> Result<Option<i32>> {
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};

        let pid = Pid::from_raw(pid as i32);
        match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, code)) => Ok(Some(code)),
            Ok(WaitStatus::Signaled(_, sig, _)) => Ok(Some(-(sig as i32))),
            Ok(WaitStatus::StillAlive) => Ok(None),
            Ok(_) => Ok(None),
            Err(nix::errno::Errno::ECHILD) => {
                // Not our child or already reaped
                if self.is_alive(pid.as_raw() as u32).await {
                    Ok(None)
                } else {
                    Ok(Some(-1))
                }
            }
            Err(e) => Err(SyspulseError::Process(format!(
                "waitpid failed for {}: {}",
                pid, e
            ))),
        }
    }

    async fn resource_usage(&self, pid: u32) -> Result<ResourceUsage> {
        let sys_pid = sysinfo::Pid::from_u32(pid);
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_processes(ProcessRefreshKind::nothing().with_memory().with_cpu()),
        );
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[sys_pid]),
            true,
            ProcessRefreshKind::nothing().with_memory().with_cpu(),
        );

        match sys.process(sys_pid) {
            Some(proc) => Ok(ResourceUsage {
                memory_bytes: proc.memory(),
                cpu_percent: proc.cpu_usage() as f64,
            }),
            None => Err(SyspulseError::Process(format!("Process {} not found", pid))),
        }
    }
}
