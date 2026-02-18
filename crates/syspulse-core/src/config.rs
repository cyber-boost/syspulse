use serde::Deserialize;

use crate::daemon::DaemonSpec;
use crate::error::{Result, SyspulseError};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ConfigFile {
    Single { daemon: DaemonSpec },
    Multi { daemon: Vec<DaemonSpec> },
}

pub fn parse_config(content: &str) -> Result<Vec<DaemonSpec>> {
    let config: ConfigFile =
        toml::from_str(content).map_err(|e| SyspulseError::Config(e.to_string()))?;
    match config {
        ConfigFile::Single { daemon } => Ok(vec![daemon]),
        ConfigFile::Multi { daemon } => Ok(daemon),
    }
}

pub fn parse_config_file(path: &std::path::Path) -> Result<Vec<DaemonSpec>> {
    let content = std::fs::read_to_string(path)?;
    parse_config(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_daemon_config() {
        let toml = r#"
[daemon]
name = "my-api"
command = ["python", "app.py"]
"#;
        let specs = parse_config(toml).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "my-api");
        assert_eq!(specs[0].command, vec!["python", "app.py"]);
    }

    #[test]
    fn parse_multi_daemon_config() {
        let toml = r#"
[[daemon]]
name = "api"
command = ["python", "api.py"]

[[daemon]]
name = "worker"
command = ["python", "worker.py"]
"#;
        let specs = parse_config(toml).unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].name, "api");
        assert_eq!(specs[1].name, "worker");
    }

    #[test]
    fn parse_config_with_all_fields() {
        let toml = r#"
[daemon]
name = "full-daemon"
command = ["node", "server.js"]
working_dir = "/opt/app"
description = "A full daemon example"
stop_timeout_secs = 60
tags = ["web", "production"]
schedule = "0 0 * * *"
user = "www-data"

[daemon.env]
NODE_ENV = "production"
PORT = "3000"

[daemon.health_check]
type = "http"
target = "http://localhost:3000/health"
interval_secs = 15
timeout_secs = 3
retries = 5
start_period_secs = 10

[daemon.restart_policy]
policy = "on_failure"
max_retries = 5
backoff_base_secs = 2.0
backoff_max_secs = 120.0

[daemon.resource_limits]
max_memory_bytes = 536870912
max_cpu_percent = 80.0

[daemon.log_config]
max_size_bytes = 104857600
retain_count = 10
compress_rotated = true
"#;
        let specs = parse_config(toml).unwrap();
        assert_eq!(specs.len(), 1);
        let spec = &specs[0];
        assert_eq!(spec.name, "full-daemon");
        assert_eq!(spec.command, vec!["node", "server.js"]);
        assert_eq!(spec.stop_timeout_secs, 60);
        assert_eq!(spec.tags, vec!["web", "production"]);
        assert_eq!(spec.description.as_deref(), Some("A full daemon example"));
        assert_eq!(spec.user.as_deref(), Some("www-data"));
        assert_eq!(spec.env.get("NODE_ENV").map(String::as_str), Some("production"));

        let hc = spec.health_check.as_ref().unwrap();
        assert_eq!(hc.target, "http://localhost:3000/health");
        assert_eq!(hc.interval_secs, 15);
        assert_eq!(hc.retries, 5);

        let rl = spec.resource_limits.as_ref().unwrap();
        assert_eq!(rl.max_memory_bytes, Some(536870912));
        assert_eq!(rl.max_cpu_percent, Some(80.0));

        let lc = spec.log_config.as_ref().unwrap();
        assert_eq!(lc.max_size_bytes, 104857600);
        assert_eq!(lc.retain_count, 10);
        assert!(lc.compress_rotated);
    }

    #[test]
    fn parse_config_defaults() {
        let toml = r#"
[daemon]
name = "minimal"
command = ["echo", "hello"]
"#;
        let specs = parse_config(toml).unwrap();
        let spec = &specs[0];
        assert_eq!(spec.stop_timeout_secs, 30);
        assert!(spec.env.is_empty());
        assert!(spec.tags.is_empty());
        assert!(spec.health_check.is_none());
        assert!(spec.resource_limits.is_none());
        assert!(spec.log_config.is_none());
        assert!(spec.schedule.is_none());
        assert!(spec.working_dir.is_none());
        assert!(spec.description.is_none());
        assert!(spec.user.is_none());
    }

    #[test]
    fn parse_invalid_config_returns_error() {
        let toml = "this is not valid toml [[[";
        assert!(parse_config(toml).is_err());
    }

    #[test]
    fn parse_missing_required_fields_returns_error() {
        let toml = r#"
[daemon]
name = "no-command"
"#;
        assert!(parse_config(toml).is_err());
    }
}
