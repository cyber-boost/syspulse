# Why Syspulse?

A practical comparison of daemon management tools to help you pick the right one.

---

## At a glance

| | systemd | supervisord | pm2 | **Syspulse** |
|---|---|---|---|---|
| **Platforms** | Linux only | Linux, macOS | Linux, macOS, Windows | **Windows, macOS, Linux** |
| **Runtime** | Built into OS | Python | Node.js | Single Rust binary |
| **Install** | Package manager + unit files | `pip install supervisor` | `npm i -g pm2` | `cargo install syspulse-cli` |
| **Config format** | INI-style unit files | INI | JS/JSON/YAML | TOML (`.sys` files) |
| **Health checks** | Watchdog protocol | External scripts required | Basic status only | HTTP, TCP, command — built in |
| **Log handling** | `journalctl` + `logrotate` | Manual rotation | Built-in, rotates on restart | Automatic rotation + compression + retention |
| **Restart policy** | Rich `Restart=` options | `autorestart=true` + delay | `restart_delay` | Exponential backoff, max retries, cooldown |
| **Resource limits** | cgroups | None | Node process limits | CPU, memory, file descriptor caps |
| **API** | D-Bus | None | Node.js programmatic API | JSON over HTTP, CLI, Rust/Python bindings |

---

## Where Syspulse shines

**Cross-platform consistency** — Same `.sys` file, same commands on Windows, macOS, and Linux. No maintaining parallel configs for `systemd`, `launchd`, and Windows Services.

**Built-in health monitoring** — HTTP, TCP, and command-based health checks with automatic restart on failure. No external scripts or monitoring agents required.

**Unified log management** — Automatic rotation with configurable size thresholds, retention counts, and optional compression. No dependency on `logrotate` or manual cleanup.

**Programmatic control** — Stable JSON API accessible from any language. First-class Rust and Python bindings for deeper integration.

**Lightweight footprint** — Single compiled binary. No Python or Node.js runtime required on the target machine.

**Tagging and organization** — Group related daemons with tags for bulk operations across services.

---

## Common questions

**"systemd is more battle-tested."**
It is — on Linux. But systemd doesn't run on Windows or macOS. For mixed-OS environments, Syspulse provides comparable reliability from a single codebase.

**"supervisord is simple enough."**
supervisord requires a Python runtime and lacks native health checks or log rotation. You end up adding those yourself.

**"pm2 works for my Node app."**
pm2 is excellent for JavaScript workloads. It doesn't support non-Node services and adds a Node.js runtime dependency.

**"I don't need cross-platform support."**
Even on a single OS, Syspulse's built-in monitoring, restart policies, and API often replace a patchwork of systemd + journalctl + logrotate + custom scripts.

---

## Decision guide

| If you… | Consider |
|---|---|
| Only run Linux services and already use systemd | **systemd** — tight OS integration is hard to beat |
| Need a lightweight Python-based supervisor on Linux/macOS | **supervisord** — simple and proven |
| Run a purely Node.js workload | **pm2** — familiar ecosystem, purpose-built |
| Manage services across Windows, macOS, and Linux | **Syspulse** — single binary, unified config |
| Want built-in health checks, log rotation, and an API in one tool | **Syspulse** — batteries included |

---

*This comparison aims to help teams pick the right tool without bias. Every option listed here is solid for its intended use case.*
