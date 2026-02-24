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
