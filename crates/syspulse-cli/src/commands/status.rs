use std::path::Path;

use anyhow::Result;
use syspulse_core::ipc::protocol::{Request, Response};

use crate::client::CliClient;
use crate::commands::OutputFormat;
use crate::output;

pub async fn run(
    socket_path: &Path,
    name: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let client = CliClient::new(socket_path);

    let response = client
        .send(Request::Status {
            name: name.map(|s| s.to_string()),
        })
        .await?;

    CliClient::ensure_success(&response)?;

    match response {
        Response::Status { instance } => {
            println!("{}", output::format_instance(&instance, format));
        }
        Response::List { instances } => {
            println!("{}", output::format_instance_list(&instances, format));
        }
        _ => {
            println!("Unexpected response");
        }
    }

    Ok(())
}
