# CLI Reference

> **Version 0.1.0**

---

## Global options

These flags can be placed before or after any subcommand.

| Flag | Short | Default | Description |
|---|---|---|---|
| `--data-dir <PATH>` | — | platform-dependent | Override the data directory (`SYSPULSE_DATA_DIR`) |
| `--socket <PATH>` | — | platform-dependent | Override the socket/pipe path |
| `--format <FMT>` | — | `table` | Output format: `table` or `json` |
| `--no-color` | — | off | Disable colored output (respects `NO_COLOR` env var) |
| `--verbose` | `-v` | off | Debug-level output |
| `--quiet` | `-q` | off | Errors only |

```bash
syspulse --data-dir /tmp/syspulse --format json status
```

---

## Commands

### `daemon`

Run the daemon manager in the foreground. Must be running before any other command works.

```bash
syspulse daemon
```

---

### `start <NAME>`

Start a named daemon.

| Flag | Description |
|---|---|
| `--wait` | Block until the daemon reports *Running* |
| `--timeout <SECS>` | Fail after this many seconds (requires `--wait`) |

```bash
syspulse start web
syspulse start web --wait --timeout 30
```

---

### `stop <NAME>`

Stop a named daemon.

| Flag | Description |
|---|---|
| `--force` | Kill immediately, skip graceful shutdown |
| `--timeout <SECS>` | How long to wait for graceful stop |

```bash
syspulse stop web
syspulse stop web --force
syspulse stop web --timeout 10
```

---

### `restart <NAME>`

Stop then start a named daemon.

| Flag | Description |
|---|---|
| `--force` | Force kill before restarting |
| `--wait` | Block until the daemon reports *Running* after restart |

```bash
syspulse restart web
syspulse restart web --force --wait
```

---

### `status [NAME]`

Show daemon status.

Without `NAME`, displays a summary table of all daemons (name, state, PID, uptime, health, restarts). With `NAME`, shows a detailed view including timestamps, exit code, and log paths.

```bash
syspulse status
syspulse status web
```

---

### `list`

List all registered daemons in summary table format.

```bash
syspulse list
```

---

### `logs <NAME>`

View daemon logs.

| Flag | Short | Default | Description |
|---|---|---|---|
| `--lines <N>` | `-n` | `50` | Number of lines to show |
| `--stderr` | — | off | Show stderr instead of stdout |
| `--follow` | `-f` | off | Follow log output *(not yet implemented in v0.1)* |

```bash
syspulse logs web
syspulse logs web --stderr -n 20
```

---

### `add`

Register a new daemon. Two mutually exclusive modes:

**From file** — load one or more daemons from a `.sys` config file:

```bash
syspulse add --file /path/to/daemons.sys
```

**Inline** — define a single daemon directly (requires both `--name` and `--command`):

```bash
syspulse add --name web --command python -m http.server 8000
```

Inline daemons use defaults: no health check, `never` restart policy, 30s stop timeout.

> `--file` and `--name`/`--command` cannot be combined.

---

### `remove <NAME>`

Remove a daemon from the manager.

| Flag | Description |
|---|---|
| `--force` | Remove even if currently running |

```bash
syspulse remove web
syspulse remove web --force
```

---

### `init [PATH]`

Generate a template `.sys` configuration file. Defaults to `syspulse.sys` in the current directory. Fails if the target file already exists.

```bash
syspulse init
syspulse init /tmp/myconfig.sys
```

---

## Default paths

| Item | Linux / macOS | Windows |
|---|---|---|
| Data directory | `~/.syspulse` | `%LOCALAPPDATA%\syspulse` |
| Socket / pipe | `<data_dir>/syspulse.sock` | `\\.\pipe\syspulse` |
| Logs | `<data_dir>/logs/<daemon>/` | `<data_dir>\logs\<daemon>\` |
| Database | `<data_dir>/syspulse.db` | `<data_dir>\syspulse.db` |
| PID file | `<data_dir>/syspulse.pid` | `<data_dir>\syspulse.pid` |

Override the data directory with the `SYSPULSE_DATA_DIR` environment variable or `--data-dir` flag. Override the socket path with `--socket`.

---

## Environment variables

| Variable | Purpose |
|---|---|
| `SYSPULSE_DATA_DIR` | Override the default data directory |
| `NO_COLOR` | Any value disables colored output ([standard convention](https://no-color.org)) |

---

## Output formats

**`--format table`** (default) — Human-readable tables with color-coded state: green for Running/Healthy, red for Failed/Unhealthy, yellow for Starting/Stopping, cyan for Scheduled, dimmed for Stopped/Unknown.

**`--format json`** — Machine-parseable JSON. Each command emits a JSON object or array matching the structure of the table output.

---

*Flags marked as not yet implemented will return an error when used in v0.1.0.*
