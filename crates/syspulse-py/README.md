# syspulse (Python)

Python SDK for Syspulse, a cross-platform daemon/process manager.

The `syspulse` package provides Python bindings to the Syspulse core runtime so you can
manage background services from Python on Windows, macOS, and Linux.

## Install

```bash
pip install syspulse
```

## Supported Python versions

- Python 3.9+
- CPython implementation

## Quick example

```python
import syspulse

print(syspulse.__version__)
```

## What this package contains

- Native extension module built with PyO3
- Typed Python package (`py.typed`)
- Async-friendly client helpers

## Build notes

- Distributed as wheels and source distribution on PyPI
- Built from Rust sources via `maturin`
- If building locally, use Python <= 3.13 with current PyO3

##Examples 
```python
"""Example Python app that manages a process with syspulse."""

from __future__ import annotations

import sys
import time
from pathlib import Path

from syspulse import (  # type: ignore[reportMissingImports]  # pylint: disable=import-error
    Daemon,
    DaemonAlreadyExistsError,
    SyspulseClient,
)


DAEMON_NAME = "py-app-worker"
HERE = Path(__file__).resolve().parent
WORKER = HERE / "worker.py"


def ensure_manager_running(client: SyspulseClient) -> None:
    if not client.is_running():
        raise SystemExit(
            "syspulse manager is not running.\n"
            "Start it in another terminal with: syspulse daemon"
        )


def build_daemon() -> Daemon:
    return Daemon(
        name=DAEMON_NAME,
        command=[sys.executable, str(WORKER)],
        working_dir=str(HERE),
        description="Example worker started from a Python app",
        tags=["example", "python"],
    )


def main() -> None:
    daemon = build_daemon()

    with SyspulseClient() as client:
        ensure_manager_running(client)

        try:
            client.add(daemon)
            print(f"added daemon: {DAEMON_NAME}")
        except DaemonAlreadyExistsError:
            print(f"daemon already exists: {DAEMON_NAME} (reusing)")

        client.start(DAEMON_NAME, wait=True, timeout=15)
        status = client.status(DAEMON_NAME)
        print(f"started: state={status.state}, pid={status.pid}")

        time.sleep(5)
        logs = client.logs(DAEMON_NAME, lines=10)
        print("recent logs:")
        for line in logs:
            print(f"  {line}")

        client.stop(DAEMON_NAME)
        client.remove(DAEMON_NAME)
        print("stopped and removed daemon")


if __name__ == "__main__":
    main()
```
### worker example to go with above
```python 
"""Tiny worker process managed by syspulse."""

from __future__ import annotations

import signal
import time


def main() -> None:
    state = {"running": True}

    def _handle_stop(_signum: int, _frame) -> None:
        print("worker: received stop signal, exiting")
        state["running"] = False

    signal.signal(signal.SIGTERM, _handle_stop)
    signal.signal(signal.SIGINT, _handle_stop)

    print("worker: started")
    i = 0
    while state["running"]:
        i += 1
        print(f"worker: heartbeat {i}")
        time.sleep(2)

    print("worker: shutdown complete")


if __name__ == "__main__":
    main()
```
## Project links

- Repository: https://github.com/cyber-boost/syspulse
- Docs: https://github.com/cyber-boost/syspulse/tree/main/docs
- Issues: https://github.com/cyber-boost/syspulse/issues

## License

MIT
