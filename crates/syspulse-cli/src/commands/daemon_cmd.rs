use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};
use syspulse_core::manager::DaemonManager;

pub async fn run(data_dir: Option<PathBuf>) -> Result<()> {
    tracing::info!("Starting syspulse daemon manager");

    let manager = match DaemonManager::new(data_dir) {
        Ok(m) => Arc::new(m),
        Err(e) => bail!("Failed to initialize daemon manager: {}", e),
    };

    manager.run().await?;
    Ok(())
}
