use std::path::Path;

use anyhow::Result;
use syspulse_core::ipc::protocol::{Request, Response};

use crate::client::CliClient;
use crate::commands::OutputFormat;

pub async fn run(
    socket_path: &Path,
    name: &str,
    wait: bool,
    timeout: Option<u64>,
    format: &OutputFormat,
) -> Result<()> {
    let client = CliClient::new(socket_path);

    let response = client
        .send(Request::Start {
            name: name.to_string(),
            wait,
            timeout_secs: timeout,
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
            println!("{}", response_summary(&response));
        }
    }

    Ok(())
}

fn response_summary(response: &Response) -> String {
    match response {
        Response::Ok { message } => message.clone(),
        Response::Error { code, message } => format!("Error ({}): {}", code, message),
        _ => "Unexpected response".to_string(),
    }
}
