use async_trait::async_trait;
use std::path::PathBuf;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tokio::time::{sleep, Duration};
use windows::Win32::Foundation::{CloseHandle, HANDLE, STILL_ACTIVE, WAIT_OBJECT_0};
use windows::Win32::System::Console::{GenerateConsoleCtrlEvent, CTRL_BREAK_EVENT};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, TerminateProcess, WaitForSingleObject,
    CREATE_NEW_PROCESS_GROUP, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE,
};

use super::{ProcessDriver, ProcessInfo, ResourceUsage};
use crate::daemon::DaemonSpec;
use crate::error::{Result, SyspulseError};

pub struct WindowsProcessDriver;

impl WindowsProcessDriver {
    pub fn new() -> Self {
        Self
    }

    fn open_process(&self, pid: u32, access: u32) -> std::result::Result<HANDLE, SyspulseError> {
        unsafe {
            OpenProcess(
                windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS(access),
                false,
                pid,
            )
            .map_err(|e| {
                SyspulseError::Process(format!("Failed to open process {}: {}", pid, e))
            })
        }
    }
}

#[async_trait]
impl ProcessDriver for WindowsProcessDriver {
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
            .stdin(std::process::Stdio::null())
            .creation_flags(CREATE_NEW_PROCESS_GROUP.0);

        if let Some(ref dir) = spec.working_dir {
            cmd.current_dir(dir);
        }

        for (key, val) in &spec.env {
            cmd.env(key, val);
        }

        cmd.kill_on_drop(false);

        let child = cmd
            .spawn()
            .map_err(|e| SyspulseError::Process(format!("Failed to spawn process: {}", e)))?;

        let pid = child
            .id()
            .ok_or_else(|| SyspulseError::Process("Failed to get child PID".into()))?;

        // Apply resource limits via Job Object if configured
        if let Some(ref limits) = spec.resource_limits {
            if limits.max_memory_bytes.is_some() {
                if let Err(e) = apply_job_limits(pid, limits) {
                    tracing::warn!(pid, "Failed to apply job object limits: {}", e);
                }
            }
        }

        // Detach
        std::mem::forget(child);

        Ok(ProcessInfo {
            pid,
            alive: true,
        })
    }

    async fn stop(&self, pid: u32, timeout_secs: u64) -> Result<()> {
        // Send CTRL_BREAK_EVENT to the process group
        unsafe {
            if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid).is_err() {
                tracing::debug!(pid, "Failed to send CTRL_BREAK_EVENT");
                return Ok(());
            }
        }

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

        tracing::warn!(pid, "Process did not exit after CTRL_BREAK, terminating");
        self.kill(pid).await
    }

    async fn kill(&self, pid: u32) -> Result<()> {
        let handle = self.open_process(pid, PROCESS_TERMINATE.0)?;
        let result = unsafe { TerminateProcess(handle, 1) };
        unsafe {
            let _ = CloseHandle(handle);
        }
        result.map_err(|e| {
            SyspulseError::Process(format!("Failed to terminate process {}: {}", pid, e))
        })?;
        Ok(())
    }

    async fn is_alive(&self, pid: u32) -> bool {
        let handle = match self.open_process(pid, PROCESS_QUERY_INFORMATION.0) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let mut exit_code: u32 = 0;
        let alive = unsafe {
            GetExitCodeProcess(handle, &mut exit_code).is_ok()
                && exit_code == STILL_ACTIVE.0 as u32
        };
        unsafe {
            let _ = CloseHandle(handle);
        }
        alive
    }

    async fn wait(&self, pid: u32) -> Result<Option<i32>> {
        let handle = match self.open_process(pid, PROCESS_QUERY_INFORMATION.0) {
            Ok(h) => h,
            Err(_) => return Ok(Some(-1)),
        };

        let result = unsafe { WaitForSingleObject(handle, 0) };
        if result == WAIT_OBJECT_0 {
            let mut exit_code: u32 = 0;
            unsafe {
                let _ = GetExitCodeProcess(handle, &mut exit_code);
                let _ = CloseHandle(handle);
            }
            Ok(Some(exit_code as i32))
        } else {
            unsafe {
                let _ = CloseHandle(handle);
            }
            Ok(None)
        }
    }

    async fn resource_usage(&self, pid: u32) -> Result<ResourceUsage> {
        let sys_pid = sysinfo::Pid::from_u32(pid);
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_processes(
                ProcessRefreshKind::new()
                    .with_memory()
                    .with_cpu(),
            ),
        );
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[sys_pid]),
            true,
            ProcessRefreshKind::new()
                .with_memory()
                .with_cpu(),
        );

        match sys.process(sys_pid) {
            Some(proc) => Ok(ResourceUsage {
                memory_bytes: proc.memory(),
                cpu_percent: proc.cpu_usage() as f64,
            }),
            None => Err(SyspulseError::Process(format!(
                "Process {} not found",
                pid
            ))),
        }
    }
}

fn apply_job_limits(pid: u32, limits: &crate::resources::ResourceLimits) -> Result<()> {
    unsafe {
        let job = CreateJobObjectW(None, None)
            .map_err(|e| SyspulseError::Process(format!("CreateJobObject failed: {}", e)))?;

        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();

        if let Some(max_mem) = limits.max_memory_bytes {
            info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
            info.ProcessMemoryLimit = max_mem as usize;
        }

        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|e| {
            let _ = CloseHandle(job);
            SyspulseError::Process(format!("SetInformationJobObject failed: {}", e))
        })?;

        let handle =
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_TERMINATE, false, pid).map_err(
                |e| {
                    let _ = CloseHandle(job);
                    SyspulseError::Process(format!("OpenProcess failed: {}", e))
                },
            )?;

        let result = AssignProcessToJobObject(job, handle);
        let _ = CloseHandle(handle);

        result.map_err(|e| {
            let _ = CloseHandle(job);
            SyspulseError::Process(format!("AssignProcessToJobObject failed: {}", e))
        })?;

        // Intentionally leak the job handle -- it must outlive the process to enforce limits.
        // In a full implementation, we would store this handle for later cleanup.
    }

    Ok(())
}
