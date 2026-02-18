# AGENTS.md – crates/syspulse-cli  

## OVERVIEW  
Command-line interface that drives the daemon manager.  

## STRUCTURE  
```
syspulse/
└─ crates/
   └─ syspulse-cli/
      ├─ Cargo.toml                 # crate manifest
      ├─ src/
      │  ├─ main.rs                # entry point, sets up async runtime
      │  ├─ lib.rs                 # re-exports command modules
      │  ├─ commands/
      │  │  ├─ start.rs            # `syspulse start`
      │  │  ├─ stop.rs             # `syspulse stop`
      │  │  ├─ restart.rs          # `syspulse restart`
      │  │  ├─ status.rs           # `syspulse status`
      │  │  ├─ add.rs              # `syspulse add`
      │  │  ├─ remove.rs           # `syspulse remove`
      │  │  ├─ init.rs             # `syspulse init`
      │  │  ├─ list.rs            # `syspulse list`
      │  │  ├─ logs.rs            # `syspulse logs`
      │  │  └─ mod.rs             # command registry
      │  ├─ client.rs              # IPC client
      │  ├─ output.rs              # pretty / JSON output
      │  └─ main.rs
      └─ tests/                     # inline test modules are in source files
```  

## WHERE TO LOOK  

| Feature | File(s) |
|--------|---------|
| Argument parsing | `src/commands/*.rs` (uses `clap` derives) |
| Start daemon | `src/commands/start.rs` |
| Stop daemon | `src/commands/stop.rs` |
| Restart flow | `src/commands/restart.rs` |
| Show daemon status | `src/commands/status.rs` |
| Add daemon | `src/commands/add.rs` |
| Remove daemon | `src/commands/remove.rs` |
| Initialize daemon | `src/commands/init.rs` |
| List daemons | `src/commands/list.rs` |
| Log extraction | `src/commands/logs.rs` |
| Output formatting | `src/output.rs` |
| IPC client | `src/client.rs` |

## CONVENTIONS (CLI specific)  
- Command definitions use `#[derive(Parser)]` from **clap**.  
- All public functions are `pub async fn run(...) -> Result<()>`.  
- Errors funnel through a single enum using `thiserror`.  
- Output can be plain text or JSON, selected by `--json` flag.  
- Code is formatted with `rustfmt` and passes `cargo clippy` without warnings.  
- Tests live inside the module they exercise, guarded by `#[cfg(test)]`.  

## ANTI-PATTERNS  
- Avoid duplicated `println!` calls for error paths; use the unified error printer.  
- Do not import `std::fs` directly for config files; rely on the helper in `syspulse-core`.  
- Never ignore the result of a `Result` from a command execution.  
- Refrain from mixing synchronous and asynchronous I/O in the same function.  
- Skip `unwrap()` on command line arguments; let `clap` handle validation.  
- Do not place long-running work in the main thread; delegate to a Tokio task.
