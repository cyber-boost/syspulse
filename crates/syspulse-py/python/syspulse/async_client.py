"""Async wrapper around :class:`SyspulseClient`.

Uses :func:`asyncio.to_thread` (Python 3.9+) to delegate every IPC call
to the synchronous Rust client in a thread-pool worker, keeping the
event loop free.
"""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING, List, Optional

from syspulse._syspulse import SyspulseClient

if TYPE_CHECKING:
    from syspulse._syspulse import Daemon, DaemonInstance


class AsyncSyspulseClient:
    """Async context-manager wrapper around the synchronous IPC client.

    Every method mirrors :class:`SyspulseClient` but returns an awaitable.

    Example::

        async with AsyncSyspulseClient() as client:
            instances = await client.list()
            for inst in instances:
                print(inst.name, inst.state)
    """

    def __init__(self, socket_path: Optional[str] = None) -> None:
        self._client = SyspulseClient(socket_path)

    # -- async context manager ------------------------------------------------

    async def __aenter__(self) -> "AsyncSyspulseClient":
        return self

    async def __aexit__(
        self,
        exc_type: object,
        exc_val: object,
        exc_tb: object,
    ) -> bool:
        return False

    # -- daemon lifecycle -----------------------------------------------------

    async def start(
        self,
        name: str,
        *,
        wait: Optional[bool] = None,
        timeout: Optional[int] = None,
    ) -> str:
        return await asyncio.to_thread(
            self._client.start, name, wait=wait, timeout=timeout
        )

    async def stop(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
        timeout: Optional[int] = None,
    ) -> str:
        return await asyncio.to_thread(
            self._client.stop, name, force=force, timeout=timeout
        )

    async def restart(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
        wait: Optional[bool] = None,
    ) -> str:
        return await asyncio.to_thread(
            self._client.restart, name, force=force, wait=wait
        )

    # -- queries --------------------------------------------------------------

    async def status(self, name: str) -> "DaemonInstance":
        return await asyncio.to_thread(self._client.status, name)

    async def list(self) -> "List[DaemonInstance]":
        return await asyncio.to_thread(self._client.list)

    async def logs(
        self,
        name: str,
        *,
        lines: Optional[int] = None,
        stderr: Optional[bool] = None,
    ) -> List[str]:
        return await asyncio.to_thread(
            self._client.logs, name, lines=lines, stderr=stderr
        )

    # -- management -----------------------------------------------------------

    async def add(self, daemon: "Daemon") -> str:
        return await asyncio.to_thread(self._client.add, daemon)

    async def remove(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
    ) -> str:
        return await asyncio.to_thread(
            self._client.remove, name, force=force
        )

    # -- health ---------------------------------------------------------------

    async def is_running(self) -> bool:
        return await asyncio.to_thread(self._client.is_running)

    async def ping(self) -> bool:
        return await asyncio.to_thread(self._client.ping)

    async def shutdown(self) -> str:
        return await asyncio.to_thread(self._client.shutdown)
