# Quick Start Guide

Get Syspulse running in under five minutes.

---

## Prerequisites

- **Rust toolchain** with `cargo` — install from [rustup.rs](https://rustup.rs)
- A terminal on Windows, macOS, or Linux

---

## 1. Install the CLI

```bash
cargo install syspulse-cli
```

After installation, ensure `~/.cargo/bin` is on your `PATH`. Most systems configure this automatically; Windows adds it to your user profile during Rust setup.

---

## 2. Create a daemon configuration

Generate a starter `.sys` file:

```bash
syspulse init mydaemons.sys
```

Open `mydaemons.sys` and replace its contents with a minimal example:

```toml
[[daemon]]
name = "example"
command = ["echo", "Hello from Syspulse"]

[daemon.restart_policy]
policy = "on_failure"
```

> **Note:** Use the singular `[[daemon]]` — not `[[daemons]]`.

---

## 3. Start the daemon manager

```bash
syspulse daemon &
```

The `&` backgrounds the process on Unix shells. On Windows PowerShell, use `Start-Process` or `Start-Job` instead.

---

## 4. Register and run your daemon

```bash
# Register the daemon(s) defined in your .sys file
syspulse add --file mydaemons.sys

# Start a specific daemon
syspulse start example

# Check what's running
syspulse status

# View log output
syspulse logs example

# Stop the daemon
syspulse stop example
```

---

## Troubleshooting

**`cargo` not found** — Ensure the Rust toolchain is installed and `~/.cargo/bin` is on your `PATH`.

**Permission errors** — On Windows, run the terminal as Administrator. On macOS/Linux, verify the target command is executable.

**Configuration errors** — Validate your `.sys` file against the [Configuration Reference →](CONFIG.md).

**Daemon exits immediately** — Check the command itself. If the process finishes quickly, the manager will restart it according to the configured restart policy.

---

## Next steps

- Add health checks, restart policies, and resource limits — see the [Configuration Reference →](CONFIG.md)
- Explore every CLI command and flag — see the [CLI Reference →](CLI.md)
- Browse ready-to-use examples in the [`examples/`](../examples/) directory
