use std::path::Path;

use anyhow::{bail, Result};

const TEMPLATE: &str = r#"# syspulse daemon configuration
# See https://github.com/user/syspulse for full documentation.

# Single daemon: use [daemon]
# Multiple daemons: use [[daemon]]

[[daemon]]
name = "my-app"
command = ["node", "server.js"]
# working_dir = "/opt/my-app"
# description = "My application server"
# user = "www-data"

# Environment variables
# [daemon.env]
# NODE_ENV = "production"
# PORT = "3000"

# Restart policy: "never" (default), "always", or "on_failure"
# [daemon.restart_policy]
# policy = "on_failure"
# max_retries = 5
# backoff_base_secs = 1.0
# backoff_max_secs = 300.0

# Health check (http, tcp, or command)
# [daemon.health_check]
# type = "http"
# target = "http://localhost:3000/health"
# interval_secs = 30
# timeout_secs = 5
# retries = 3
# start_period_secs = 10

# Resource limits
# [daemon.resource_limits]
# max_memory_bytes = 536870912  # 512 MB
# max_cpu_percent = 80.0

# Log rotation
# [daemon.log_config]
# max_size_bytes = 52428800  # 50 MB
# retain_count = 5
# compress_rotated = false

# Cron-style schedule (optional)
# schedule = "0 0 * * *"

# Tags for grouping
# tags = ["web", "production"]

# Graceful stop timeout in seconds (default: 30)
# stop_timeout_secs = 30
"#;

pub fn run(path: &Path) -> Result<()> {
    if path.exists() {
        bail!(
            "File '{}' already exists. Remove it first or choose a different path.",
            path.display()
        );
    }

    std::fs::write(path, TEMPLATE)?;
    println!("Created template config at: {}", path.display());
    println!(
        "Edit this file and then run: syspulse add --file {}",
        path.display()
    );

    Ok(())
}
