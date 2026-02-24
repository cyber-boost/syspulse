use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use anyhow::{bail, Result};
use syspulse_core::config::parse_config_file;
use syspulse_core::daemon::DaemonSpec;
use syspulse_core::ipc::protocol::{Request, Response};
use syspulse_core::restart::RestartPolicy;

use crate::client::CliClient;
use crate::commands::OutputFormat;

const DEFAULT_CONFIG_CANDIDATES: [&str; 2] = ["syspulse.sys", "syspulse.toml"];

fn has_supported_config_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("sys") | Some("toml")
    )
}

fn discover_config_file_in_dir(dir: &Path) -> Result<Option<std::path::PathBuf>> {
    for candidate in DEFAULT_CONFIG_CANDIDATES {
        let path = dir.join(candidate);
        if path.is_file() {
            return Ok(Some(path));
        }
    }

    let mut matching_files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && has_supported_config_extension(&path) {
            matching_files.push(path);
        }
    }

    match matching_files.len() {
        0 => Ok(None),
        1 => Ok(matching_files.pop()),
        _ => {
            matching_files.sort();
            let choices = matching_files
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n");
            bail!(
                "Multiple config files found in '{}'. Use --file to choose one:\n{}",
                dir.display(),
                choices
            );
        }
    }
}

fn discover_config_file() -> Result<Option<std::path::PathBuf>> {
    let cwd = std::env::current_dir()?;
    discover_config_file_in_dir(&cwd)
}

pub async fn run(
    socket_path: &Path,
    file: Option<&Path>,
    name: Option<&str>,
    command: Option<&[String]>,
    format: &OutputFormat,
) -> Result<()> {
    let discovered_file = if file.is_none() && name.is_none() && command.is_none() {
        discover_config_file()?
    } else {
        None
    };

    let config_file = file.map(Path::to_path_buf).or(discovered_file);

    let specs = match config_file.as_deref() {
        Some(path) => parse_config_file(path)?,
        None => {
            let name = name.ok_or_else(|| {
                anyhow::anyhow!("Either --file or --name and --command are required")
            })?;
            let command = command
                .ok_or_else(|| anyhow::anyhow!("--command is required when using --name"))?;
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
                eprintln!(
                    "Failed to add '{}': Error ({}): {}",
                    spec_name, code, message
                );
            }
            _ => {
                eprintln!("Unexpected response for '{}'", spec_name);
            }
        }
    }

    if added == 0 && config_file.is_some() {
        bail!("No daemons were added");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_dir(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("syspulse-add-tests-{}-{}", name, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn prefers_syspulse_sys_when_present() {
        let dir = temp_test_dir("prefers-sys");
        let sys = dir.join("syspulse.sys");
        let toml = dir.join("syspulse.toml");

        std::fs::write(&sys, "[daemon]\nname='x'\ncommand=['echo','x']\n").unwrap();
        std::fs::write(&toml, "[daemon]\nname='y'\ncommand=['echo','y']\n").unwrap();

        let discovered = discover_config_file_in_dir(&dir).unwrap();
        assert_eq!(discovered.as_deref(), Some(sys.as_path()));

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn discovers_single_sys_file() {
        let dir = temp_test_dir("single-sys");
        let sys = dir.join("custom.sys");

        std::fs::write(&sys, "[daemon]\nname='x'\ncommand=['echo','x']\n").unwrap();

        let discovered = discover_config_file_in_dir(&dir).unwrap();
        assert_eq!(discovered.as_deref(), Some(sys.as_path()));

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn errors_when_multiple_candidates_exist() {
        let dir = temp_test_dir("multiple-candidates");
        std::fs::write(
            dir.join("alpha.sys"),
            "[daemon]\nname='x'\ncommand=['echo','x']\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("beta.toml"),
            "[daemon]\nname='y'\ncommand=['echo','y']\n",
        )
        .unwrap();

        let err = discover_config_file_in_dir(&dir).unwrap_err().to_string();
        assert!(err.contains("Multiple config files found"));

        std::fs::remove_dir_all(dir).unwrap();
    }
}
