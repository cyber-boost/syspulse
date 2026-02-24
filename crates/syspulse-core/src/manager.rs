use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{error, info, warn};

use crate::daemon::{DaemonInstance, DaemonSpec, HealthStatus};
use crate::error::{Result, SyspulseError};
use crate::ipc::protocol::{Request, Response};
use crate::ipc::server::IpcServer;
use crate::lifecycle::LifecycleState;
use crate::logs::LogManager;
use crate::paths;
use crate::process::{self, ProcessDriver};
use crate::registry::Registry;
use crate::restart::RestartEvaluator;
use crate::scheduler::Scheduler;

pub struct DaemonManager {
    registry: Arc<Mutex<Registry>>,
    process_driver: Arc<dyn ProcessDriver>,
    log_manager: Arc<LogManager>,
    instances: Arc<RwLock<HashMap<String, DaemonInstance>>>,
    health_handles: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl DaemonManager {
    /// Create a new DaemonManager. If `data_dir` is None, uses the default.
    pub fn new(data_dir: Option<PathBuf>) -> Result<Self> {
        let data = data_dir.unwrap_or_else(paths::data_dir);
        paths::ensure_dirs()?;

        let db_path = data.join("syspulse.db");
        let registry = Registry::new(&db_path)?;
        let process_driver = process::create_driver();
        let log_manager = LogManager::new(&data);
        let (shutdown_tx, _) = broadcast::channel(16);

        // Load existing instance states from the registry.
        let saved_states = registry.list_states().unwrap_or_default();
        let mut instances = HashMap::new();
        for inst in saved_states {
            instances.insert(inst.spec_name.clone(), inst);
        }

        Ok(Self {
            registry: Arc::new(Mutex::new(registry)),
            process_driver: Arc::from(process_driver),
            log_manager: Arc::new(log_manager),
            instances: Arc::new(RwLock::new(instances)),
            health_handles: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx,
        })
    }

    /// Start a daemon by name.
    pub async fn start_daemon(&self, name: &str) -> Result<DaemonInstance> {
        let spec = {
            let reg = self.registry.lock().await;
            reg.get_spec(name)?
        };

        // Get or create the instance.
        let mut instances = self.instances.write().await;
        let instance = instances
            .entry(name.to_string())
            .or_insert_with(|| DaemonInstance::new(name));

        // Validate state transition.
        let new_state = instance.state.transition_to(LifecycleState::Starting)?;
        instance.state = new_state;

        // Set up log files.
        let (stdout_path, stderr_path) = self.log_manager.setup_log_files(name)?;
        instance.stdout_log = Some(stdout_path.clone());
        instance.stderr_log = Some(stderr_path.clone());

        // Spawn the process.
        let proc_info = self
            .process_driver
            .spawn(&spec, &stdout_path, &stderr_path)
            .await?;

        instance.pid = Some(proc_info.pid);
        instance.started_at = Some(Utc::now());
        instance.stopped_at = None;
        instance.exit_code = None;
        instance.state = instance.state.transition_to(LifecycleState::Running)?;

        if spec.health_check.is_some() {
            instance.health_status = HealthStatus::Unknown;
        } else {
            instance.health_status = HealthStatus::NotConfigured;
        }

        // Persist state.
        {
            let reg = self.registry.lock().await;
            reg.update_state(instance)?;
        }

        let result = instance.clone();

        // Drop the write lock before spawning health check.
        drop(instances);

        // Start health check background task if configured.
        if let Some(ref health_spec) = spec.health_check {
            let daemon_name = name.to_string();
            let shutdown_rx = self.shutdown_tx.subscribe();
            let instances = Arc::clone(&self.instances);
            let registry = Arc::clone(&self.registry);
            let health_spec: crate::daemon::HealthCheckSpec = health_spec.clone();

            let handle = tokio::spawn(async move {
                Self::run_health_check(instances, registry, daemon_name, health_spec, shutdown_rx)
                    .await;
            });

            let mut handles = self.health_handles.lock().await;
            handles.insert(name.to_string(), handle);
        }

        info!(
            "Started daemon '{}' with PID {}",
            name,
            result.pid.unwrap_or(0)
        );
        Ok(result)
    }

    /// Stop a running daemon.
    pub async fn stop_daemon(&self, name: &str, force: bool) -> Result<DaemonInstance> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(name)
            .ok_or_else(|| SyspulseError::DaemonNotFound(name.to_string()))?;

        if !instance.state.is_active() {
            return Err(SyspulseError::InvalidStateTransition {
                from: format!("{:?}", instance.state),
                to: "Stopping".to_string(),
            });
        }

        instance.state = instance.state.transition_to(LifecycleState::Stopping)?;

        // Cancel health check task.
        {
            let mut handles = self.health_handles.lock().await;
            if let Some(handle) = handles.remove(name) {
                handle.abort();
            }
        }

        // Stop the process.
        if let Some(pid) = instance.pid {
            if force {
                self.process_driver.kill(pid).await?;
            } else {
                let timeout = {
                    let reg = self.registry.lock().await;
                    reg.get_spec(name)
                        .map(|s| s.stop_timeout_secs)
                        .unwrap_or(30)
                };
                self.process_driver.stop(pid, timeout).await?;
            }
            // Try to get exit code.
            let exit_code = self.process_driver.wait(pid).await.ok().flatten();
            instance.exit_code = exit_code;
        }

        instance.state = instance.state.transition_to(LifecycleState::Stopped)?;
        instance.stopped_at = Some(Utc::now());
        instance.pid = None;
        instance.health_status = HealthStatus::Unknown;

        // Persist state.
        {
            let reg = self.registry.lock().await;
            reg.update_state(instance)?;
        }

        info!("Stopped daemon '{}'", name);
        Ok(instance.clone())
    }

    /// Restart a daemon (stop then start).
    pub async fn restart_daemon(&self, name: &str, force: bool) -> Result<DaemonInstance> {
        // Only stop if active.
        {
            let instances = self.instances.read().await;
            if let Some(inst) = instances.get(name) {
                if inst.state.is_active() {
                    drop(instances);
                    self.stop_daemon(name, force).await?;
                }
            }
        }
        self.start_daemon(name).await
    }

    /// Get the current status of a daemon.
    pub async fn status(&self, name: &str) -> Result<DaemonInstance> {
        let instances = self.instances.read().await;
        instances
            .get(name)
            .cloned()
            .ok_or_else(|| SyspulseError::DaemonNotFound(name.to_string()))
    }

    /// List all daemon instances.
    pub async fn list(&self) -> Result<Vec<DaemonInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.values().cloned().collect())
    }

    /// Register a new daemon spec.
    pub async fn add_daemon(&self, spec: DaemonSpec) -> Result<()> {
        let name = spec.name.clone();

        {
            let reg = self.registry.lock().await;
            reg.register(&spec)?;
        }

        // Initialize instance in Stopped state (or Scheduled if it has a cron).
        let mut instance = DaemonInstance::new(&name);
        if spec.schedule.is_some() {
            instance.state = LifecycleState::Scheduled;
        }

        let mut instances = self.instances.write().await;
        instances.insert(name.clone(), instance);

        info!("Added daemon '{}'", name);
        Ok(())
    }

    /// Remove a daemon. If `force` is true, stop it first if running.
    pub async fn remove_daemon(&self, name: &str, force: bool) -> Result<()> {
        // Check if running.
        {
            let instances = self.instances.read().await;
            if let Some(inst) = instances.get(name) {
                if inst.state.is_active() {
                    if !force {
                        return Err(SyspulseError::Process(format!(
                            "Daemon '{}' is still running. Use force to stop and remove.",
                            name
                        )));
                    }
                }
            }
        }

        // Stop if active and force.
        if force {
            let instances = self.instances.read().await;
            let is_active = instances
                .get(name)
                .map(|i| i.state.is_active())
                .unwrap_or(false);
            drop(instances);
            if is_active {
                self.stop_daemon(name, true).await?;
            }
        }

        // Unregister.
        {
            let reg = self.registry.lock().await;
            reg.unregister(name)?;
        }

        // Remove from in-memory map.
        {
            let mut instances = self.instances.write().await;
            instances.remove(name);
        }

        // Cancel any health check.
        {
            let mut handles = self.health_handles.lock().await;
            if let Some(handle) = handles.remove(name) {
                handle.abort();
            }
        }

        info!("Removed daemon '{}'", name);
        Ok(())
    }

    /// Read logs for a daemon.
    pub async fn get_logs(&self, name: &str, lines: usize, stderr: bool) -> Result<Vec<String>> {
        // Verify the daemon exists.
        {
            let instances = self.instances.read().await;
            if !instances.contains_key(name) {
                return Err(SyspulseError::DaemonNotFound(name.to_string()));
            }
        }
        self.log_manager.read_logs(name, lines, stderr)
    }

    /// Dispatch an IPC request to the appropriate method and return a response.
    pub async fn handle_request(self: &Arc<Self>, request: Request) -> Response {
        match request {
            Request::Start { name, .. } => match self.start_daemon(&name).await {
                Ok(inst) => Response::Ok {
                    message: format!("Daemon '{}' started (PID {})", name, inst.pid.unwrap_or(0)),
                },
                Err(e) => error_response(e),
            },
            Request::Stop { name, force, .. } => match self.stop_daemon(&name, force).await {
                Ok(_) => Response::Ok {
                    message: format!("Daemon '{}' stopped", name),
                },
                Err(e) => error_response(e),
            },
            Request::Restart { name, force, .. } => match self.restart_daemon(&name, force).await {
                Ok(inst) => Response::Ok {
                    message: format!(
                        "Daemon '{}' restarted (PID {})",
                        name,
                        inst.pid.unwrap_or(0)
                    ),
                },
                Err(e) => error_response(e),
            },
            Request::Status { name } => match name {
                Some(name) => match self.status(&name).await {
                    Ok(instance) => Response::Status { instance },
                    Err(e) => error_response(e),
                },
                None => match self.list().await {
                    Ok(instances) => Response::List { instances },
                    Err(e) => error_response(e),
                },
            },
            Request::List => match self.list().await {
                Ok(instances) => Response::List { instances },
                Err(e) => error_response(e),
            },
            Request::Logs {
                name,
                lines,
                stderr,
            } => match self.get_logs(&name, lines, stderr).await {
                Ok(log_lines) => Response::Logs { lines: log_lines },
                Err(e) => error_response(e),
            },
            Request::Add { spec } => match self.add_daemon(spec).await {
                Ok(()) => Response::Ok {
                    message: "Daemon added".to_string(),
                },
                Err(e) => error_response(e),
            },
            Request::Remove { name, force } => match self.remove_daemon(&name, force).await {
                Ok(()) => Response::Ok {
                    message: format!("Daemon '{}' removed", name),
                },
                Err(e) => error_response(e),
            },
            Request::Shutdown => {
                info!("Shutdown requested via IPC");
                // The actual shutdown is triggered by the caller seeing this response.
                // We signal the broadcast channel.
                let _ = self.shutdown_tx.send(());
                Response::Ok {
                    message: "Shutting down".to_string(),
                }
            }
            Request::Ping => Response::Pong,
        }
    }

    /// Main entry point for the daemon manager. Called by `syspulse daemon`.
    pub async fn run(self: Arc<Self>) -> Result<()> {
        info!("Starting syspulse daemon manager");

        // Write PID file.
        let pid_path = paths::pid_path();
        std::fs::write(&pid_path, std::process::id().to_string())?;

        // Restore daemons that were Running before a crash/restart.
        self.restore_running_daemons().await;

        // Set up cron scheduler for scheduled daemons.
        let mut scheduler = Scheduler::new().await?;
        self.setup_scheduled_daemons(&mut scheduler).await?;
        scheduler.start().await?;

        // Start the IPC server.
        let socket_path = paths::socket_path();
        let ipc_server = IpcServer::new(socket_path);
        let shutdown_rx_ipc = self.shutdown_tx.subscribe();

        let manager_for_ipc = Arc::clone(&self);
        let ipc_handle = tokio::spawn(async move {
            let handler = Arc::new(move |req: Request| {
                let mgr = Arc::clone(&manager_for_ipc);
                async move { mgr.handle_request(req).await }
            });
            if let Err(e) = ipc_server.run(handler, shutdown_rx_ipc).await {
                error!("IPC server error: {}", e);
            }
        });

        // Start the process monitor background task.
        let manager_for_monitor = Arc::clone(&self);
        let shutdown_rx_monitor = self.shutdown_tx.subscribe();
        let monitor_handle = tokio::spawn(async move {
            Self::monitor_processes(manager_for_monitor, shutdown_rx_monitor).await;
        });

        // Wait for shutdown signal (Ctrl+C / SIGTERM).
        let shutdown_tx = self.shutdown_tx.clone();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, initiating shutdown");
                let _ = shutdown_tx.send(());
            }
            _ = self.wait_for_shutdown() => {
                info!("Shutdown signal received");
            }
        }

        // Graceful shutdown: stop all running daemons.
        info!("Stopping all running daemons...");
        self.stop_all_daemons().await;

        // Shut down scheduler.
        scheduler.shutdown().await.ok();

        // Wait for background tasks to finish.
        ipc_handle.abort();
        monitor_handle.abort();
        let _ = tokio::join!(ipc_handle, monitor_handle);

        // Clean up PID file.
        std::fs::remove_file(&pid_path).ok();

        info!("Daemon manager shut down cleanly");
        Ok(())
    }

    /// Wait until a shutdown signal is received on the broadcast channel.
    async fn wait_for_shutdown(&self) {
        let mut rx = self.shutdown_tx.subscribe();
        let _ = rx.recv().await;
    }

    /// Attempt to restore daemons that were in Running state when we last shut down.
    async fn restore_running_daemons(&self) {
        let instances = self.instances.read().await;
        let to_restart: Vec<String> = instances
            .iter()
            .filter(|(_, inst)| inst.state == LifecycleState::Running)
            .map(|(name, _)| name.clone())
            .collect();
        drop(instances);

        for name in to_restart {
            // First mark as stopped (the old process is gone), then start fresh.
            {
                let mut instances = self.instances.write().await;
                if let Some(inst) = instances.get_mut(&name) {
                    inst.state = LifecycleState::Stopped;
                    inst.pid = None;
                }
            }
            info!("Restoring previously running daemon '{}'", name);
            if let Err(e) = self.start_daemon(&name).await {
                error!("Failed to restore daemon '{}': {}", name, e);
            }
        }
    }

    /// Set up cron schedules for all daemons that have a schedule field.
    async fn setup_scheduled_daemons(&self, scheduler: &mut Scheduler) -> Result<()> {
        let specs = {
            let reg = self.registry.lock().await;
            reg.list_specs().unwrap_or_default()
        };

        for spec in specs {
            if let Some(ref cron_expr) = spec.schedule {
                let manager = Arc::new({
                    // We need a reference to self for the callback, but we're behind Arc<Self>
                    // in run(). The callback captures the instances/registry/etc. via clones.
                    let instances = Arc::clone(&self.instances);
                    let registry = Arc::clone(&self.registry);
                    let process_driver = Arc::clone(&self.process_driver);
                    let log_manager = Arc::clone(&self.log_manager);
                    let shutdown_tx = self.shutdown_tx.clone();
                    let health_handles = Arc::clone(&self.health_handles);
                    ManagerComponents {
                        instances,
                        registry,
                        process_driver,
                        log_manager,
                        shutdown_tx,
                        health_handles,
                    }
                });

                scheduler
                    .schedule_daemon(&spec.name, cron_expr, move |name| {
                        let mgr = Arc::clone(&manager);
                        async move {
                            info!("Cron trigger: starting daemon '{}'", name);
                            if let Err(e) = cron_start_daemon(&mgr, &name).await {
                                error!("Cron failed to start '{}': {}", name, e);
                            }
                        }
                    })
                    .await?;
            }
        }

        Ok(())
    }

    /// Stop all currently running daemons (used during shutdown).
    async fn stop_all_daemons(&self) {
        let names: Vec<String> = {
            let instances = self.instances.read().await;
            instances
                .iter()
                .filter(|(_, inst)| inst.state.is_active())
                .map(|(name, _)| name.clone())
                .collect()
        };

        for name in names {
            if let Err(e) = self.stop_daemon(&name, false).await {
                warn!("Failed to stop daemon '{}' during shutdown: {}", name, e);
                // Try force kill.
                if let Err(e2) = self.stop_daemon(&name, true).await {
                    error!("Failed to force-stop daemon '{}': {}", name, e2);
                }
            }
        }
    }

    /// Background task: monitors running processes, detects unexpected exits,
    /// and handles restart policies.
    async fn monitor_processes(
        manager: Arc<DaemonManager>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = shutdown_rx.recv() => {
                    info!("Process monitor shutting down");
                    break;
                }
            }

            // Collect names of daemons in Running state.
            let running: Vec<(String, u32)> = {
                let instances = manager.instances.read().await;
                instances
                    .iter()
                    .filter_map(|(name, inst)| {
                        if inst.state == LifecycleState::Running {
                            inst.pid.map(|pid| (name.clone(), pid))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            for (name, pid) in running {
                let alive = manager.process_driver.is_alive(pid).await;
                if alive {
                    continue;
                }

                // Process has exited unexpectedly.
                warn!("Daemon '{}' (PID {}) has exited unexpectedly", name, pid);

                let exit_code = manager.process_driver.wait(pid).await.ok().flatten();

                // Update instance state.
                let (should_restart, backoff) = {
                    let mut instances = manager.instances.write().await;
                    if let Some(inst) = instances.get_mut(&name) {
                        inst.state = LifecycleState::Failed;
                        inst.pid = None;
                        inst.exit_code = exit_code;
                        inst.stopped_at = Some(Utc::now());
                        inst.health_status = HealthStatus::Unknown;

                        // Persist the failed state.
                        if let Ok(reg) = manager.registry.try_lock() {
                            reg.update_state(inst).ok();
                        }

                        // Check restart policy.
                        let spec = {
                            if let Ok(reg) = manager.registry.try_lock() {
                                reg.get_spec(&name).ok()
                            } else {
                                None
                            }
                        };

                        if let Some(spec) = spec {
                            let should = RestartEvaluator::should_restart(
                                &spec.restart_policy,
                                exit_code,
                                inst.restart_count,
                            );
                            let backoff = RestartEvaluator::backoff_duration(
                                &spec.restart_policy,
                                inst.restart_count,
                            );
                            inst.restart_count += 1;
                            (should, backoff)
                        } else {
                            (false, Duration::ZERO)
                        }
                    } else {
                        (false, Duration::ZERO)
                    }
                };

                // Cancel health check.
                {
                    let mut handles = manager.health_handles.lock().await;
                    if let Some(handle) = handles.remove(&name) {
                        handle.abort();
                    }
                }

                if should_restart {
                    info!("Restarting daemon '{}' after {:?} backoff", name, backoff);

                    let mgr = Arc::clone(&manager);
                    let daemon_name = name.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(backoff).await;
                        // Reset state to Stopped so we can transition to Starting.
                        {
                            let mut instances = mgr.instances.write().await;
                            if let Some(inst) = instances.get_mut(&daemon_name) {
                                inst.state = LifecycleState::Stopped;
                            }
                        }
                        if let Err(e) = mgr.start_daemon(&daemon_name).await {
                            error!("Failed to restart daemon '{}': {}", daemon_name, e);
                        }
                    });
                }
            }
        }
    }

    /// Background task: runs periodic health checks for a daemon.
    async fn run_health_check(
        instances: Arc<RwLock<HashMap<String, DaemonInstance>>>,
        registry: Arc<Mutex<Registry>>,
        daemon_name: String,
        health_spec: crate::daemon::HealthCheckSpec,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        use crate::health;

        // Wait for the start period before beginning checks.
        if health_spec.start_period_secs > 0 {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(health_spec.start_period_secs)) => {}
                _ = shutdown_rx.recv() => return,
            }
        }

        let checker = health::create_checker(health_spec.clone());
        let interval = Duration::from_secs(health_spec.interval_secs);
        let max_failures = health_spec.retries;
        let mut consecutive_failures: u32 = 0;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = shutdown_rx.recv() => break,
            }

            let result = checker.check().await;
            let status = match result {
                Ok(s) => s,
                Err(e) => {
                    warn!("Health check error for '{}': {}", daemon_name, e);
                    HealthStatus::Unhealthy
                }
            };

            match status {
                HealthStatus::Healthy => {
                    consecutive_failures = 0;
                    let mut insts = instances.write().await;
                    if let Some(inst) = insts.get_mut(&daemon_name) {
                        if inst.state == LifecycleState::Running {
                            inst.health_status = HealthStatus::Healthy;
                        }
                    }
                }
                HealthStatus::Unhealthy => {
                    consecutive_failures += 1;
                    if consecutive_failures >= max_failures {
                        warn!(
                            "Daemon '{}' is unhealthy after {} consecutive failures",
                            daemon_name, consecutive_failures
                        );
                        let mut insts = instances.write().await;
                        if let Some(inst) = insts.get_mut(&daemon_name) {
                            inst.health_status = HealthStatus::Unhealthy;
                            if let Ok(reg) = registry.try_lock() {
                                reg.update_state(inst).ok();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Internal helper struct for passing manager components into cron callbacks
/// without requiring Arc<DaemonManager>.
struct ManagerComponents {
    instances: Arc<RwLock<HashMap<String, DaemonInstance>>>,
    registry: Arc<Mutex<Registry>>,
    process_driver: Arc<dyn ProcessDriver>,
    log_manager: Arc<LogManager>,
    shutdown_tx: broadcast::Sender<()>,
    health_handles: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

/// Start a daemon using raw components (for cron callbacks).
async fn cron_start_daemon(components: &ManagerComponents, name: &str) -> Result<DaemonInstance> {
    let spec = {
        let reg = components.registry.lock().await;
        reg.get_spec(name)?
    };

    let mut instances = components.instances.write().await;
    let instance = instances
        .entry(name.to_string())
        .or_insert_with(|| DaemonInstance::new(name));

    // For scheduled daemons, allow Scheduled -> Starting or Stopped -> Starting.
    if instance.state == LifecycleState::Scheduled || instance.state == LifecycleState::Stopped {
        instance.state = instance.state.transition_to(LifecycleState::Starting)?;
    } else if instance.state == LifecycleState::Running {
        // Already running, nothing to do.
        return Ok(instance.clone());
    } else {
        return Err(SyspulseError::InvalidStateTransition {
            from: format!("{:?}", instance.state),
            to: "Starting".to_string(),
        });
    }

    let (stdout_path, stderr_path) = components.log_manager.setup_log_files(name)?;
    instance.stdout_log = Some(stdout_path.clone());
    instance.stderr_log = Some(stderr_path.clone());

    let proc_info = components
        .process_driver
        .spawn(&spec, &stdout_path, &stderr_path)
        .await?;

    instance.pid = Some(proc_info.pid);
    instance.started_at = Some(Utc::now());
    instance.stopped_at = None;
    instance.exit_code = None;
    instance.state = instance.state.transition_to(LifecycleState::Running)?;

    if spec.health_check.is_some() {
        instance.health_status = HealthStatus::Unknown;
    } else {
        instance.health_status = HealthStatus::NotConfigured;
    }

    {
        let reg = components.registry.lock().await;
        reg.update_state(instance)?;
    }

    let result = instance.clone();
    drop(instances);

    // Start health check if configured.
    if let Some(ref health_spec) = spec.health_check {
        let daemon_name = name.to_string();
        let shutdown_rx = components.shutdown_tx.subscribe();
        let insts = Arc::clone(&components.instances);
        let registry = Arc::clone(&components.registry);
        let hs: crate::daemon::HealthCheckSpec = health_spec.clone();

        let handle = tokio::spawn(async move {
            DaemonManager::run_health_check(insts, registry, daemon_name, hs, shutdown_rx).await;
        });

        let mut handles = components.health_handles.lock().await;
        handles.insert(name.to_string(), handle);
    }

    info!(
        "Cron-started daemon '{}' with PID {}",
        name,
        result.pid.unwrap_or(0)
    );
    Ok(result)
}

fn error_response(e: SyspulseError) -> Response {
    let code = match &e {
        SyspulseError::DaemonNotFound(_) => 404,
        SyspulseError::DaemonAlreadyExists(_) => 409,
        SyspulseError::InvalidStateTransition { .. } => 409,
        SyspulseError::Process(_) => 500,
        SyspulseError::HealthCheck(_) => 500,
        SyspulseError::Ipc(_) => 500,
        SyspulseError::Registry(_) => 500,
        SyspulseError::Config(_) => 400,
        SyspulseError::Scheduler(_) => 500,
        SyspulseError::Io(_) => 500,
        SyspulseError::Serialization(_) => 400,
        SyspulseError::Database(_) => 500,
        SyspulseError::Timeout(_) => 504,
    };
    Response::Error {
        code,
        message: e.to_string(),
    }
}
