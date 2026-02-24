# Python app example (using `syspulse`)

This folder shows how to use the `syspulse` Python package inside a normal Python app.

## What this example does

- Connects to a running syspulse manager
- Registers a daemon that runs `worker.py`
- Starts it, checks status, prints recent logs
- Stops and removes it (cleanup)

## Prerequisites

1. Install/build the Python package

   From repo root:

   ```powershell
   cd crates/syspulse-py
   pip install maturin
   maturin develop
   ```

2. Start the syspulse manager in another terminal

   ```powershell
   syspulse daemon
   ```

## Run

From repo root:

```powershell
python examples/py-app/app.py
```

You should see daemon status and log lines from `worker.py`.

## Files

- `app.py` - main app using `SyspulseClient`
- `worker.py` - simple long-running process managed by syspulse
