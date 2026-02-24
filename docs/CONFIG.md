# syspulse Configuration Reference (v0.1.0)

## Daemon specification

The top‑level daemon description lives under `[daemon]` for a single daemon or `[[daemon]]` for an array of daemons. All keys listed below are parsed directly from the `DaemonSpec` struct.

| TOML key | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | String | yes | — | Unique identifier for the daemon |
| `command` | Array of String | yes | — | Executable and arguments to run |
| `working_dir` | String (path) | no | current directory | Working directory for the process |
| `env` | Table of String→String | no | empty | Environment variables passed to the process |
| `health_check` | Table | no | none | Configuration for health monitoring |
| `restart_policy` | Table | no | `never` | Restart behaviour after exit |
| `resource_limits` | Table | no | none | Optional resource caps |
| `schedule` | String | no | none | Cron expression; if present the daemon runs on schedule instead of continuously |
| `tags` | Array of String | no | empty | Arbitrary tags for grouping |
| `stop_timeout_secs` | Integer | no | `30` | Seconds to wait for graceful shutdown before killing |
| `log_config` | Table | no | none (defaults when section present) | Log rotation settings |
| `description` | String | no | none | Human‑readable description |
| `user` | String | no | none | Unix user to run as (Unix only) |

## Health check specification

Defined under `[daemon.health_check]`. All keys mirror the `HealthCheckSpec` struct.

| TOML key | Type | Required | Default | Description |
|---|---|---|---|---|
| `type` | "http", "tcp", "command" | yes | — | Kind of health probe |
| `target` | String | yes | — | Target string, format depends on `type` |
| `interval_secs` | Integer | no | `30` | Seconds between successive checks |
| `timeout_secs` | Integer | no | `5` | Seconds before a single check times out |
| `retries` | Integer | no | `3` | Failures required to mark daemon unhealthy |
| `start_period_secs` | Integer | no | `0` | Grace period after start before the first check |

### Health check types
- **http** – `target` must be a URL, e.g. `"http://127.0.0.1:3000/health"`. An HTTP GET is issued; any 2xx response is considered healthy.
- **tcp** – `target` is `"host:port"`, e.g. `"127.0.0.1:5432"`. A TCP connection attempt is made; success marks the daemon healthy.
- **command** – `target` is a shell command, e.g. `"pg_isready -h localhost"`. The command runs; exit code `0` means healthy.

## Restart policy specification

The `[daemon.restart_policy]` table is a tagged enum keyed by `policy`.

### `policy = "never"` (default)
The daemon is not restarted after exit. No additional fields.

### `policy = "always"`
Restart the daemon after every exit, regardless of exit code.

### `policy = "on_failure"`
Restart only when the exit code is non-zero or the process was killed by a signal.

### Backoff fields (apply to `always` and `on_failure` only)

| TOML key | Type | Default | Description |
|---|---|---|---|
| `max_retries` | Integer (optional) | unlimited | Maximum restart attempts; omit for unlimited |
| `backoff_base_secs` | Float | `1.0` | Initial backoff delay in seconds |
| `backoff_max_secs` | Float | `300.0` | Upper bound for backoff delay |

Delay is calculated as `backoff_base_secs * 2^attempt`, capped at `backoff_max_secs`, plus 0-10% random jitter.

## Resource limits

Optional `[daemon.resource_limits]` section.

| TOML key | Type | Default | Description |
|---|---|---|---|
| `max_memory_bytes` | Integer | none | Upper memory bound in bytes |
| `max_cpu_percent` | Float | none | Maximum CPU usage as a percentage (0‑100) |
| `max_open_files` | Integer | none | Upper limit on open file descriptors |

## Log configuration

Optional `[daemon.log_config]` section. Fields have built‑in defaults.

| TOML key | Type | Default | Description |
|---|---|---|---|
| `max_size_bytes` | Integer | `52428800` (50 MB) | Size at which the log file rotates |
| `retain_count` | Integer | `5` | How many rotated files to keep |
| `compress_rotated` | Boolean | `false` | Whether to gzip rotated files |

## Environment variables

Environment variables are expressed as a table under `[daemon.env]`.
```toml
[daemon.env]
NODE_ENV = "production"
PORT = "3000"
```
Each key/value pair is passed to the daemon unchanged.

## Single vs. multiple daemon syntax
- **Single daemon** – use a single table `[daemon]`. The file can contain only one daemon configuration.
- **Multiple daemons** – use an array of tables `[[daemon]]`. Each block defines an independent daemon. The parser treats both forms identically.

## Example configurations

### Minimal example
```toml
[daemon]
name = "echo-server"
command = ["python", "-m", "http.server", "8000"]
```

### Web server with health check
```toml
[daemon]
name = "web-api"
command = ["python", "-m", "uvicorn", "app:main", "--port", "8080"]
working_dir = "/app"
description = "Main web API server"

[daemon.health_check]
type = "http"
target = "http://127.0.0.1:8080/health"
interval_secs = 30
timeout_secs = 5
retries = 3

[daemon.restart_policy]
policy = "on_failure"
max_retries = 5
backoff_base_secs = 1
backoff_max_secs = 300
```

### Background worker (no health check, always restart)
```toml
[daemon]
name = "worker"
command = ["/usr/local/bin/worker", "--queue", "jobs"]

[daemon.restart_policy]
policy = "always"
backoff_base_secs = 2
backoff_max_secs = 120
```

### Scheduled job
```toml
[daemon]
name = "cleanup-job"
command = ["python", "scripts/cleanup.py"]
schedule = "0 0 * * *"
description = "Daily cleanup job"

[daemon.restart_policy]
policy = "never"
```

## Further reading
All example files mentioned above live in the `examples/` directory of the repository.

---
*Version 0.1.0*