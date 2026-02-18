# AGENTS.md – crates/syspulse-core

**OVERVIEW**  
Core daemon logic for process lifecycle, health monitoring, and IPC.

**STRUCTURE**  
```
syspulse-core/
├─ src/
│  ├─ lib.rs                 # crate root, re-exports
│  ├─ manager.rs             # high-level daemon manager
│  ├─ config.rs              # TOML configuration parsing
│  ├─ process/
│  │  ├─ mod.rs
│  │  ├─ unix.rs            # Unix-specific process handling
│  │  └─ windows.rs         # Windows-specific process handling
│  ├─ health/
│  │  ├─ mod.rs
│  │  ├─ http.rs            # HTTP health checks
│  │  ├─ tcp.rs             # TCP health checks
│  │  └─ command.rs         # Exec-based health checks
│  ├─ ipc/
│  │  ├─ mod.rs
│  │  └─ protocol.rs        # IPC message definitions
│  └─ utils.rs               # helpers used across modules
└─ Cargo.toml                 # crate manifest
```

**WHERE TO LOOK**  

| Area                     | Path                                   |
|--------------------------|----------------------------------------|
| Daemon manager          | `crates/syspulse-core/src/manager.rs` |
| Config handling          | `crates/syspulse-core/src/config.rs` |
| Platform-specific code   | `crates/syspulse-core/src/process/*` |
| Health check implementations | `crates/syspulse-core/src/health/*` |
| IPC protocol            | `crates/syspulse-core/src/ipc/protocol.rs` |
| Utility helpers          | `crates/syspulse-core/src/utils.rs`   |

**CONVENTIONS**  
* Errors use `thiserror` together with `anyhow` for context.  
* Async functions return `Result<_, anyhow::Error>` and are driven by Tokio.  
* Platform modules expose a unified `ProcessHandler` trait; each file implements the appropriate OS version.  
* Public API is re-exported from `lib.rs` to keep the crate surface small.

**ANTI-PATTERNS**  
No hidden globals, unsafe blocks are confined to OS-specific modules and fully reviewed. No `unwrap` or `expect` in production code; all fallible calls propagate errors. No duplicated health-check logic—each check type lives in its own module and shares a common trait.
