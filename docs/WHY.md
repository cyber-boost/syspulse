# Why Choose syspulse?

When managing background services or daemons, several tools are commonly considered: **systemd**, **supervisord**, **pm2**, and the newer **syspulse**. Each has its own strengths and trade‑offs. The table below summarizes the key dimensions that influence the decision.

| Feature | systemd | supervisord | pm2 | syspulse |
|---|---|---|---|---|
| **Platform** | Linux only | Linux/macOS (Python) | Linux/macOS/Windows (Node.js) | Windows, macOS, Linux (single Rust binary) |
| **Installation complexity** | Packages & unit files; steep learning curve | `pip install supervisor`; simple INI files | `npm i -g pm2`; JavaScript‑centric config | Cargo binary or pre‑built release; minimal config files |
| **Health monitoring** | Native `systemd` watchdog, `systemctl status` | Requires external scripts | Basic `pm2 status`; optional modules | Built‑in HTTP/TCP/command checks; customizable probes |
| **Log handling** | `journalctl` or separate files; rotation via `logrotate` | Manual rotation scripts | Integrated log files; rotates on restart | Automatic rotation with compression and retention policies |
| **Restart policy** | Rich `Restart=` options, exponential backoff via `RestartSec` | `autorestart=true`; simple delay | `restart_delay` option; limited policies | Smart policies with exponential backoff, max retries, cooldown periods |
| **Resource limits** | cgroups (`MemoryLimit=`, `CPUQuota=`) | No built‑in limits | Limited to Node process limits | CPU & memory caps, OOM handling, optional cgroup integration |
| **IPC / API** | D‑Bus, `systemctl` CLI | No native API | `pm2` programmatic API (Node) | JSON over HTTP, CLI, and Rust/Python bindings |
| **Dependency footprint** | Part of system; heavy but stable | Python runtime | Node.js runtime | Single compiled binary + optional Python bindings |
| **Use case focus** | System services, boot‑time daemons | Simple process supervision | Node.js applications | Cross‑platform services, mixed language stacks, unified ops tooling |

## When syspulse shines
- **Cross‑platform consistency**: You need the same tool on Windows, macOS, and Linux without maintaining separate configurations.
- **Built‑in health checks**: Services must report health via HTTP, TCP, or custom commands, and you want automatic restart on failure.
- **Unified logging**: Automatic rotation, compression, and retention keep logs tidy without external tools.
- **Programmatic control**: Scripts in Rust, Python, or any language can manage daemons through a stable JSON API.
- **Lightweight footprint**: No need for a full Python or Node.js runtime if you already ship a Rust binary.
- **Tagging & organization**: Group related daemons with tags for bulk operations (start/stop/status).

## Common objections addressed
- **"systemd is more battle‑tested"** – systemd excels on Linux servers, but it does not run on Windows or macOS. For mixed‑OS fleets, syspulse provides comparable reliability with a single code base.
- **"supervisord is simple enough"** – supervisord requires a Python interpreter and lacks native health checks or log rotation, which you would have to add yourself.
- **"pm2 works for my Node app"** – pm2 is great for JavaScript, but it does not support non‑Node services and adds a Node runtime dependency.
- **"I don’t need cross‑platform support"** – Even on a single OS, syspulse’s built‑in monitoring, restart policies, and API often replace a collection of separate tools (systemd + journalctl + custom scripts).

## Decision guidance
1. **If you only run Linux services and already rely on systemd**, stick with systemd – it integrates tightly with the OS.
2. **If you need a lightweight, Python‑based supervisor for a few processes on Linux/macOS**, supervisord remains a solid choice.
3. **If your workload is purely Node.js and you are comfortable with the Node ecosystem**, pm2 gives you a familiar workflow.
4. **If you manage services across Windows, macOS, and Linux, or you want a single binary with health checks, logging, and an API**, syspulse is the most balanced solution.

---
*This document aims to help teams pick the right tool for their service‑management needs without bias.*
