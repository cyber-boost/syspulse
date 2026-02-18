use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

use crate::error::{Result, SyspulseError};

pub struct Scheduler {
    scheduler: JobScheduler,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| SyspulseError::Scheduler(e.to_string()))?;
        Ok(Self { scheduler })
    }

    /// Schedule a daemon to be triggered on a cron expression.
    /// The callback receives the daemon name and should start the daemon.
    pub async fn schedule_daemon<F, Fut>(
        &mut self,
        name: &str,
        cron_expr: &str,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let daemon_name = name.to_string();
        let cb = callback.clone();
        let job = Job::new_async(cron_expr, move |_uuid, _lock| {
            let name = daemon_name.clone();
            let cb = cb.clone();
            Box::pin(async move {
                info!("Cron trigger firing for daemon '{}'", name);
                cb(name).await;
            })
        })
        .map_err(|e| SyspulseError::Scheduler(e.to_string()))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| SyspulseError::Scheduler(e.to_string()))?;
        info!("Scheduled daemon '{}' with cron '{}'", name, cron_expr);
        Ok(())
    }

    /// Start the scheduler so jobs begin firing.
    pub async fn start(&self) -> Result<()> {
        self.scheduler
            .start()
            .await
            .map_err(|e| SyspulseError::Scheduler(e.to_string()))?;
        info!("Cron scheduler started");
        Ok(())
    }

    /// Shut down the scheduler, stopping all jobs.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| SyspulseError::Scheduler(e.to_string()))?;
        info!("Cron scheduler shut down");
        Ok(())
    }
}
