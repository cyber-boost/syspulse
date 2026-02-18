# PROJECT KNOWLEDGE BASE

**Generated:** 2026-02-18
**Branch:** main (no commits)

## OVERVIEW
Daemon manager for Unix/Windows systems. Manages process lifecycle (start, stop, restart), health monitoring, and IPC. Rust monorepo with 3 crates + Python bindings.

## STRUCTURE
```
syspulse/
├── Cargo.toml           # Workspace root
├── crates/
│   ├── syspulse-core/  # Core daemon management logic
│   ├── syspulse-cli/   # CLI application
│   └── syspulse-python/ # Python bindings (pyo3)
└── examples/           # TOML config examples
```

## WHERE TO LOOK
| Task | Location |
|------|----------|
| Core daemon logic | crates/syspulse-core/src/manager.rs |
| CLI commands | crates/syspulse-cli/src/commands/ |
| Health checks | crates/syspulse-core/src/health/ |
| IPC protocol | crates/syspulse-core/src/ipc/ |
| Process management | crates/syspulse-core/src/process/ |
| Config parsing | crates/syspulse-core/src/config.rs |

## CONVENTIONS
- Rust 2021 edition, workspace resolver v2
- Inline tests only (`#[cfg(test)] mod tests` in source files)
- No separate `tests/` directory
- Error handling: `thiserror` + `anyhow`
- Async runtime: `tokio` with full features

## ANTI-PATTERNS (THIS PROJECT)
- None detected in source code (generated bindgen constants excluded)

## UNIQUE STYLES
- Platform-specific process handling: `unix.rs` + `windows.rs` in same crate
- TOML-based daemon configuration
- Health check types: HTTP, TCP, command execution

## COMMANDS
```bash
cargo build          # Build all crates
cargo test          # Run inline tests
cargo run -p syspulse-cli -- --help  # CLI help
```

## NOTES
- No CI/CD configured (no .github/workflows, no Makefile)
- SQLite bundled via rusqlite for state storage
- Python bindings use maturin build system
