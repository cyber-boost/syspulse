"""Type stubs for syspulse Python bindings."""

from enum import IntEnum
from typing import Dict, List, Optional

__version__: str

class DaemonStatus(IntEnum):
    Stopped = 0
    Starting = 1
    Running = 2
    Stopping = 3
    Failed = 4
    Scheduled = 5

class HealthStatus(IntEnum):
    Unknown = 0
    Healthy = 1
    Unhealthy = 2
    NotConfigured = 3

class RestartPolicyType(IntEnum):
    Always = 0
    OnFailure = 1
    Never = 2

class Daemon:
    @property
    def name(self) -> str: ...
    @property
    def command(self) -> List[str]: ...
    @property
    def working_dir(self) -> Optional[str]: ...
    @property
    def env(self) -> Dict[str, str]: ...
    @property
    def schedule(self) -> Optional[str]: ...
    @property
    def tags(self) -> List[str]: ...
    @property
    def stop_timeout(self) -> int: ...
    @property
    def description(self) -> Optional[str]: ...
    @property
    def restart_policy(self) -> RestartPolicyType: ...

    def __init__(
        self,
        name: str,
        command: List[str],
        *,
        working_dir: Optional[str] = None,
        env: Optional[Dict[str, str]] = None,
        schedule: Optional[str] = None,
        tags: Optional[List[str]] = None,
        stop_timeout: int = 30,
        description: Optional[str] = None,
    ) -> None: ...
    def __repr__(self) -> str: ...
    def with_health_check(
        self,
        check_type: str,
        target: str,
        *,
        interval: Optional[int] = None,
        timeout: Optional[int] = None,
        retries: Optional[int] = None,
        start_period: Optional[int] = None,
    ) -> Daemon: ...
    def with_restart_policy(
        self,
        policy: str,
        *,
        max_retries: Optional[int] = None,
        backoff_base: Optional[float] = None,
        backoff_max: Optional[float] = None,
    ) -> Daemon: ...

class SyspulseClient:
    def __init__(self, socket_path: Optional[str] = None) -> None: ...
    def start(
        self,
        name: str,
        *,
        wait: Optional[bool] = None,
        timeout: Optional[int] = None,
    ) -> str: ...
    def stop(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
        timeout: Optional[int] = None,
    ) -> str: ...
    def restart(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
        wait: Optional[bool] = None,
    ) -> str: ...
    def status(self, name: str) -> Dict[str, object]: ...
    def list(self) -> List[Dict[str, object]]: ...
    def logs(
        self,
        name: str,
        *,
        lines: Optional[int] = None,
        stderr: Optional[bool] = None,
    ) -> List[str]: ...
    def add(self, daemon: Daemon) -> str: ...
    def remove(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
    ) -> str: ...
    def is_running(self) -> bool: ...
    def ping(self) -> bool: ...
    def shutdown(self) -> str: ...

def from_toml(path: str) -> List[Daemon]: ...
