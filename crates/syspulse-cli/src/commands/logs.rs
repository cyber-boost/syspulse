use std::path::Path;

use anyhow::{bail, Result};
use syspulse_core::ipc::protocol::{Request, Response};

use crate::client::CliClient;
use crate::commands::OutputFormat;

pub async fn run(
    socket_path: &Path,
    name: &str,
    lines: usize,
    stderr: bool,
    follow: bool,
    format: &OutputFormat,
) -> Result<()> {
    if follow {
        bail!("--follow is not yet implemented in v0.1");
    }

    let client = CliClient::new(socket_path);

    let response = client
        .send(Request::Logs {
            name: name.to_string(),
            lines,
            stderr,
        })
        .await?;

    CliClient::ensure_success(&response)?;

    match (&response, format) {
        (Response::Logs { lines }, OutputFormat::Table) => {
            for line in lines {
                println!("{}", line);
            }
        }
        (Response::Logs { lines }, OutputFormat::Json) => {
            println!("{}", serde_json::to_string_pretty(lines)?);
        }
        _ => {
            println!("Unexpected response");
        }
    }

    Ok(())
}
