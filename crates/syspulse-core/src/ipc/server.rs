use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

use crate::error::{Result, SyspulseError};
use crate::ipc::protocol::{read_message, write_message, Request, Response};

#[cfg(unix)]
use interprocess::local_socket::GenericFilePath as NameType;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced as NameType;

use interprocess::local_socket::{
    traits::tokio::Listener, tokio::prelude::*, ListenerOptions,
};

pub struct IpcServer {
    socket_path: PathBuf,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Run the IPC server, dispatching each request to the given handler.
    ///
    /// The handler receives a `Request` and returns a `Response`. The server
    /// keeps running until the handler returns a response indicating shutdown
    /// or `shutdown_rx` fires.
    pub async fn run<F, Fut>(
        &self,
        handler: Arc<F>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()>
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send,
    {
        // On Unix, remove stale socket file if it exists.
        #[cfg(unix)]
        {
            if self.socket_path.exists() {
                std::fs::remove_file(&self.socket_path).ok();
            }
        }

        let name = self.socket_name()?;
        let listener = ListenerOptions::new()
            .name(name)
            .create_tokio()
            .map_err(|e| SyspulseError::Ipc(format!("Failed to create listener: {}", e)))?;

        info!("IPC server listening on {:?}", self.socket_path);

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok(stream) => {
                            let handler = Arc::clone(&handler);
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, handler).await {
                                    warn!("IPC connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept IPC connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("IPC server shutting down");
                    break;
                }
            }
        }

        // Cleanup socket file on Unix
        #[cfg(unix)]
        {
            std::fs::remove_file(&self.socket_path).ok();
        }

        Ok(())
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
            // On Windows, use a named pipe. The socket_path is something like
            // \\.\pipe\syspulse, but interprocess expects just the name part.
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

async fn handle_connection<F, Fut>(
    stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    handler: Arc<F>,
) -> Result<()>
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Response> + Send,
{
    let (mut reader, mut writer) = tokio::io::split(stream);

    // Handle multiple requests per connection until the client disconnects.
    loop {
        let request: Option<Request> = read_message(&mut reader).await?;
        let request = match request {
            Some(r) => r,
            None => break, // Client disconnected
        };

        let is_shutdown = matches!(request, Request::Shutdown);
        let response = handler(request).await;
        write_message(&mut writer, &response).await?;
        writer.flush().await?;

        if is_shutdown {
            break;
        }
    }

    Ok(())
}
