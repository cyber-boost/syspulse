use std::path::PathBuf;

use tracing::debug;

use crate::error::{Result, SyspulseError};
use crate::ipc::protocol::{read_message, write_message, Request, Response};

#[cfg(unix)]
use interprocess::local_socket::GenericFilePath as NameType;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced as NameType;

use interprocess::local_socket::tokio::prelude::*;

pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Send a request to the daemon manager and return the response.
    /// Each call creates a fresh connection (simple request-response model).
    pub async fn send(&self, request: Request) -> Result<Response> {
        let name = self.socket_name()?;
        let stream = LocalSocketStream::connect(name)
            .await
            .map_err(|e| SyspulseError::Ipc(format!("Failed to connect: {}", e)))?;

        let (mut reader, mut writer) = tokio::io::split(stream);

        write_message(&mut writer, &request).await?;
        debug!("Sent IPC request: {:?}", request);

        let response: Response = read_message(&mut reader)
            .await?
            .ok_or_else(|| SyspulseError::Ipc("Server closed connection without response".into()))?;

        debug!("Received IPC response: {:?}", response);
        Ok(response)
    }

    /// Check if the daemon manager is reachable by sending a Ping.
    pub async fn is_manager_running(&self) -> bool {
        match self.send(Request::Ping).await {
            Ok(Response::Pong) => true,
            _ => false,
        }
    }

    fn socket_name(&self) -> Result<interprocess::local_socket::Name<'_>> {
        #[cfg(unix)]
        {
            let path_str = self
                .socket_path
                .to_str()
                .ok_or_else(|| SyspulseError::Ipc("Invalid socket path".into()))?;
            path_str
                .to_ns_name::<NameType>()
                .map_err(|e| SyspulseError::Ipc(format!("Invalid socket name: {}", e)))
        }
        #[cfg(windows)]
        {
            let name_str = self
                .socket_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("syspulse");
            name_str
                .to_ns_name::<NameType>()
                .map_err(|e| SyspulseError::Ipc(format!("Invalid pipe name: {}", e)))
        }
    }
}
