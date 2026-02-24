# syspulse CLI Reference (v0.1.0)

The `syspulse` binary provides a set of commands to manage daemons, inspect their status, and configure the system. All commands share a set of **global options** that can be placed before or after the sub‑command.

---

## Global Options

| Flag | Short | Type | Default | Env Var | Description |
|------|-------|------|---------|---------|-------------|
| `--data-dir` | — | PATH | platform‑dependent | `SYSPULSE_DATA_DIR` | Override the default data directory |
| `--socket` | — | PATH | platform‑dependent | — | Override the socket/pipe path |
| `--format` | — | `table` \| `json` | `table` | — | Choose output format |
| `--no-color` | — | flag | off | respects `NO_COLOR` | Disable colored output |
| `--verbose` | `-v` | flag | off | — | Debug‑level output |
| `--quiet` | `-q` | flag | off | — | Show errors only |

### Example
```bash
syspulse --data-dir /tmp/syspulse --format json status
```

---

## Commands

### `daemon`
Run the daemon manager in the foreground. No additional arguments. The daemon must be running before any other command works.

```bash
syspulse daemon
```

---

### `start <NAME>`
Start a named daemon.

| Flag | Type | Description |
|------|------|-------------|
| `--wait` | flag | Block until the daemon reports *Running* |
| `--timeout <SECS>` | u64 (optional) | Timeout in seconds; only used with `--wait` |

**Examples**
```bash
# Start "web" and return immediately
syspulse start web

# Block until "web" is running, fail after 30 s
syspulse start web --wait --timeout 30
```

---

### `stop <NAME>`
Stop a named daemon.

| Flag | Type | Description |
|------|------|-------------|
| `--force` | flag | Kill the process immediately, skipping graceful shutdown |
| `--timeout <SECS>` | u64 (optional) | How long to wait for a graceful stop |

**Examples**
```bash
# Graceful stop, default timeout
syspulse stop web

# Force kill
syspulse stop web --force

# Graceful stop with a 10 s timeout
syspulse stop web --timeout 10
```

---

### `restart <NAME>`
Restart a named daemon (stop then start).

| Flag | Type | Description |
|------|------|-------------|
| `--force` | flag | Force kill before restarting |
| `--wait` | flag | Block until the daemon reports *Running* after restart |

**Examples**
```bash
# Simple restart
syspulse restart web

# Restart and wait for it to be ready
syspulse restart web --wait

# Force‑kill then restart, waiting for readiness
syspulse restart web --force --wait
```

---

### `status [NAME]`
Show daemon status.
- Without `NAME`: table of all daemons (Name, State, PID, Uptime, Health, Restarts).
- With `NAME`: detailed view including ID, State, PID, Health, Restarts, Uptime, start/stop timestamps, exit code, and log paths.

```bash
# All daemons
syspulse status

# Detailed view for "web"
syspulse status web
```

---

### `list`
List all registered daemons in the same table format as `status` (no detailed view).

```bash
syspulse list
```

---

### `logs <NAME>`
View daemon logs.

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `-n, --lines <N>` | — | usize | 50 | Number of lines to show |
| `--stderr` | — | flag | off | Show *stderr* instead of *stdout* |
| `-f, --follow` | — | flag | off | Follow log output ( **not yet implemented in v0.1** ) |

**Examples**
```bash
# Show last 50 lines of stdout (default)
syspulse logs web

# Show last 20 lines of stderr
syspulse logs web --stderr -n 20

# Attempt to follow (will error in v0.1)
syspulse logs web --follow
```

---

### `add`
Add a new daemon. Two mutually exclusive modes:

1. **File mode** – Load daemon specifications from a `.sys` config file. Multiple daemons can be defined in one file.
   ```bash
   syspulse add --file /path/to/daemons.sys
   ```
2. **Inline mode** – Define a single daemon directly. Each word after `--command` becomes a separate argument.
   ```bash
   syspulse add --name web --command python -m http.server 8000
   ```
   Both `--name` **and** `--command` must be supplied together. The inline daemon is created with defaults: no health check, `never` restart policy, 30 s stop timeout.

*Note*: `--file` cannot be used together with `--name`/`--command`.

---

### `remove <NAME>`
Remove a daemon from the manager.

| Flag | Type | Description |
|------|------|-------------|
| `--force` | flag | Remove even if the daemon is currently running |

```bash
# Simple removal
syspulse remove web

# Force removal while running
syspulse remove web --force
```

---

### `init [PATH]`
Generate a template `.sys` configuration file.
- `PATH` defaults to `syspulse.sys` in the current directory.
- The command fails if the target file already exists.

```bash
# Create default config in current directory
syspulse init

# Write to a custom location
syspulse init /tmp/myconfig.sys
```

---

## Default Paths (platform‑specific)

| Item | Linux/macOS | Windows |
|------|--------------|----------|
| Data directory | `~/.syspulse` | `%LOCALAPPDATA%\syspulse` |
| Socket / pipe | `<data_dir>/syspulse.sock` | `\\.\pipe\syspulse` |
| Logs | `<data_dir>/logs/<daemon-name>/` | `<data_dir>\logs\<daemon-name>\` |
| Database | `<data_dir>/syspulse.db` | `<data_dir>\syspulse.db` |
| PID file | `<data_dir>/syspulse.pid` | `<data_dir>\syspulse.pid` |

Overrides: use `SYSPULSE_DATA_DIR` env var or `--data-dir` flag for the data directory; use `--socket` for the socket/pipe path.

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `SYSPULSE_DATA_DIR` | Override the default data directory |
| `NO_COLOR` | Any value disables colored output (standard convention) |

---

## Output Formats

- `--format table` (default): Human‑readable tables with colors indicating state (green = Running/Healthy, red = Failed/Unhealthy, yellow = Starting/Stopping, cyan = Scheduled, dimmed = Stopped/Unknown).
- `--format json`: Machine‑parseable JSON; each command emits a JSON object or array matching the textual output.

---

*All information reflects the functionality present in version **0.1.0** of `syspulse`. Flags marked as not yet implemented will return an error when used.*
