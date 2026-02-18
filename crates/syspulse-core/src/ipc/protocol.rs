use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::daemon::{DaemonInstance, DaemonSpec};
use crate::error::SyspulseError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Start {
        name: String,
        wait: bool,
        timeout_secs: Option<u64>,
    },
    Stop {
        name: String,
        force: bool,
        timeout_secs: Option<u64>,
    },
    Restart {
        name: String,
        force: bool,
        wait: bool,
    },
    Status {
        name: Option<String>,
    },
    List,
    Logs {
        name: String,
        lines: usize,
        stderr: bool,
    },
    Add {
        spec: DaemonSpec,
    },
    Remove {
        name: String,
        force: bool,
    },
    Shutdown,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Ok { message: String },
    Status { instance: DaemonInstance },
    List { instances: Vec<DaemonInstance> },
    Logs { lines: Vec<String> },
    Pong,
    Error { code: u32, message: String },
}

/// Encode a message as 4-byte big-endian length prefix + JSON payload.
pub fn encode_message<T: Serialize>(msg: &T) -> crate::error::Result<Vec<u8>> {
    let json = serde_json::to_vec(msg)?;
    let len = (json.len() as u32).to_be_bytes();
    let mut buf = Vec::with_capacity(4 + json.len());
    buf.extend_from_slice(&len);
    buf.extend_from_slice(&json);
    Ok(buf)
}

/// Read a length-prefixed JSON message from an async reader.
/// Returns `Ok(None)` on clean EOF (peer disconnected).
pub async fn read_message<T: serde::de::DeserializeOwned>(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> crate::error::Result<Option<T>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 10 * 1024 * 1024 {
        return Err(SyspulseError::Ipc("Message too large (>10MB)".into()));
    }
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    let msg = serde_json::from_slice(&payload)?;
    Ok(Some(msg))
}

/// Write a length-prefixed JSON message to an async writer.
pub async fn write_message<T: Serialize>(
    writer: &mut (impl AsyncWriteExt + Unpin),
    msg: &T,
) -> crate::error::Result<()> {
    let encoded = encode_message(msg)?;
    writer.write_all(&encoded).await?;
    writer.flush().await?;
    Ok(())
}
