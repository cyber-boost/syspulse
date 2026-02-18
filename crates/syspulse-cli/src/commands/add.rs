use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Result};
use syspulse_core::config::parse_config_file;
use syspulse_core::daemon::DaemonSpec;
use syspulse_core::ipc::protocol::{Request, Response};
use syspulse_core::restart::RestartPolicy;

use crate::client::CliClient;
use crate::commands::OutputFormat;

pub async fn run(
    socket_path: &Path,
    file: Option<&Path>,
    name: Option<&str>,
    command: Option<&[String]>,
    format: &OutputFormat,
) -> Result<()> {
    let specs = match file {
        Some(path) => parse_config_file(path)?,
        None => {
            let name = name.ok_or_else(|| {
                anyhow::anyhow!("Either --file or --name and --command are required")
            })?;
            let command = command.ok_or_else(|| {
                anyhow::anyhow!("--command is required when using --name")
            })?;
            if command.is_empty() {
                bail!("--command must not be empty");
            }
            vec![DaemonSpec {
                name: name.to_string(),
                command: command.to_vec(),
                working_dir: None,
                env: HashMap::new(),
                health_check: None,
                restart_policy: RestartPolicy::default(),
                resource_limits: None,
                schedule: None,
                tags: Vec::new(),
                stop_timeout_secs: 30,
                log_config: None,
                description: None,
                user: None,
            }]
        }
    };

    let client = CliClient::new(socket_path);
    let mut added = 0;

    for spec in specs {
        let spec_name = spec.name.clone();
        let response = client.send(Request::Add { spec }).await?;

        match (&response, format) {
            (Response::Ok { message }, OutputFormat::Table) => {
                println!("{}", message);
                added += 1;
            }
            (Response::Ok { message }, OutputFormat::Json) => {
                println!(
                    "{}",
                    serde_json::json!({ "status": "ok", "name": spec_name, "message": message })
                );
                added += 1;
            }
            (Response::Error { code, message }, _) => {
                eprintln!("Failed to add '{}': Error ({}): {}", spec_name, code, message);
            }
            _ => {
                eprintln!("Unexpected response for '{}'", spec_name);
            }
        }
    }

    if added == 0 && file.is_some() {
        bail!("No daemons were added");
    }

    Ok(())
}
