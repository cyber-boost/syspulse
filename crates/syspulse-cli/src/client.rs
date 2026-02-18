use std::path::Path;

use anyhow::{bail, Result};
use syspulse_core::ipc::client::IpcClient;
use syspulse_core::ipc::protocol::{Request, Response};

pub struct CliClient {
    inner: IpcClient,
}

impl CliClient {
    pub fn new(socket_path: &Path) -> Self {
        Self {
            inner: IpcClient::new(socket_path.to_path_buf()),
        }
    }

    pub async fn send(&self, request: Request) -> Result<Response> {
        if !self.inner.is_manager_running().await {
            bail!("syspulse daemon is not running. Start it with: syspulse daemon");
        }
        Ok(self.inner.send(request).await?)
    }

    pub fn ensure_success(response: &Response) -> Result<()> {
        if let Response::Error { code, message } = response {
            bail!("Error ({}): {}", code, message);
        }
        Ok(())
    }
}
