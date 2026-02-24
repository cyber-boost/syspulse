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
