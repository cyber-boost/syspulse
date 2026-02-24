# Configuration Reference

> **Version 0.1.0**

Syspulse daemons are defined in `.sys` files using TOML syntax. Use `[daemon]` for a single daemon or `[[daemon]]` for multiple daemons in one file.

---

## Daemon specification

| Key | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | String | **yes** | — | Unique identifier for the daemon |
| `command` | Array of String | **yes** | — | Executable and arguments |
| `description` | String | no | — | Human-readable description |
| `working_dir` | String (path) | no | current directory | Working directory for the process |
| `env` | Table (String → String) | no | — | Environment variables passed to the process |
| `user` | String | no | — | Unix user to run as (Unix only) |
| `tags` | Array of String | no | — | Arbitrary tags for grouping and filtering |
| `stop_timeout_secs` | Integer | no | `30` | Seconds to wait for graceful shutdown before kill |
| `schedule` | String (cron) | no | — | Cron expression; daemon runs on schedule instead of continuously |
| `health_check` | Table | no | — | Health monitoring configuration |
| `restart_policy` | Table | no | `never` | Restart behavior after exit |
| `resource_limits` | Table | no | — | Memory, CPU, and file descriptor caps |
| `log_config` | Table | no | — | Log rotation settings |

---

## Health checks

Defined under `[daemon.health_check]`.

| Key | Type | Required | Default | Description |
|---|---|---|---|---|
| `type` | `"http"` · `"tcp"` · `"command"` | **yes** | — | Kind of health probe |
| `target` | String | **yes** | — | Probe target (format depends on `type`) |
| `interval_secs` | Integer | no | `30` | Seconds between checks |
| `timeout_secs` | Integer | no | `5` | Seconds before a single check times out |
| `retries` | Integer | no | `3` | Consecutive failures before marking unhealthy |
| `start_period_secs` | Integer | no | `0` | Grace period after start before the first check |

### Check types

**`http`** — Sends an HTTP GET to `target`. Any 2xx response is healthy.
```toml
[daemon.health_check]
type = "http"
target = "http://127.0.0.1:8080/health"
```

**`tcp`** — Attempts a TCP connection to `host:port`. Connection success is healthy.
```toml
[daemon.health_check]
type = "tcp"
target = "127.0.0.1:5432"
```

**`command`** — Runs a shell command. Exit code `0` is healthy.
```toml
[daemon.health_check]
type = "command"
target = "pg_isready -h localhost"
```

---

## Restart policies

Defined under `[daemon.restart_policy]`. The `policy` key selects the strategy.

### `never` (default)

The daemon is not restarted after exit.

### `always`

Restart after every exit, regardless of exit code.

### `on_failure`

Restart only when the exit code is non-zero or the process was killed by a signal.

### Backoff options

These keys apply to `always` and `on_failure` policies:

| Key | Type | Default | Description |
|---|---|---|---|
| `max_retries` | Integer | unlimited | Maximum restart attempts (omit for unlimited) |
| `backoff_base_secs` | Float | `1.0` | Initial backoff delay in seconds |
| `backoff_max_secs` | Float | `300.0` | Upper bound for backoff delay |

Delay formula: `backoff_base_secs × 2^attempt`, capped at `backoff_max_secs`, plus 0–10% random jitter.

---

## Resource limits

Optional `[daemon.resource_limits]` section.

| Key | Type | Default | Description |
|---|---|---|---|
| `max_memory_bytes` | Integer | — | Upper memory bound in bytes |
| `max_cpu_percent` | Float | — | Maximum CPU usage (0–100) |
| `max_open_files` | Integer | — | Upper limit on open file descriptors |

---

## Log configuration

Optional `[daemon.log_config]` section.

| Key | Type | Default | Description |
|---|---|---|---|
| `max_size_bytes` | Integer | `52428800` (50 MB) | File size that triggers rotation |
| `retain_count` | Integer | `5` | Rotated files to keep |
| `compress_rotated` | Boolean | `false` | Gzip rotated files |

---

## Environment variables

Expressed as a table under `[daemon.env]`:

```toml
[daemon.env]
NODE_ENV = "production"
PORT = "3000"
```

Each key-value pair is passed to the daemon process unchanged.

---

## Examples

### Minimal

```toml
[daemon]
name = "echo-server"
command = ["python", "-m", "http.server", "8000"]
```

### Web server with health check and restart policy

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

### Background worker

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
description = "Daily cleanup at midnight"

[daemon.restart_policy]
policy = "never"
```

### Multi-daemon file

```toml
[[daemon]]
name = "api"
command = ["./api-server", "--port", "8080"]
tags = ["backend"]

[[daemon]]
name = "worker"
command = ["./worker", "--queue", "jobs"]
tags = ["backend"]

[[daemon]]
name = "scheduler"
command = ["./scheduler"]
schedule = "*/5 * * * *"
tags = ["infra"]
```

---

More examples are available in the [`examples/`](../examples/) directory.
