use std::path::Path;

use anyhow::Result;
use syspulse_core::ipc::protocol::{Request, Response};

use crate::client::CliClient;
use crate::commands::OutputFormat;

pub async fn run(
    socket_path: &Path,
    name: &str,
    force: bool,
    format: &OutputFormat,
) -> Result<()> {
    let client = CliClient::new(socket_path);

    let response = client
        .send(Request::Remove {
            name: name.to_string(),
            force,
        })
        .await?;

    CliClient::ensure_success(&response)?;

    match (&response, format) {
        (Response::Ok { message }, OutputFormat::Table) => {
            println!("{}", message);
        }
        (Response::Ok { message }, OutputFormat::Json) => {
            println!(
                "{}",
                serde_json::json!({ "status": "ok", "message": message })
            );
        }
        _ => {
            println!("Unexpected response");
        }
    }

    Ok(())
}
