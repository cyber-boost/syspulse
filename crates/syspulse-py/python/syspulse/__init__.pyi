"""Type stubs for the syspulse Python SDK."""

from enum import IntEnum
from types import TracebackType
from typing import Dict, List, Optional, Type

__version__: str

# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------

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

class HealthCheckType(IntEnum):
    Http = 0
    Tcp = 1
    Command = 2

class RestartPolicyType(IntEnum):
    Always = 0
    OnFailure = 1
    Never = 2

# ---------------------------------------------------------------------------
# Config classes
# ---------------------------------------------------------------------------

class HealthCheck:
    def __init__(
        self,
        check_type: HealthCheckType,
        target: str,
        *,
        interval: int = 30,
        timeout: int = 5,
        retries: int = 3,
        start_period: int = 0,
    ) -> None: ...
    @property
    def check_type(self) -> HealthCheckType: ...
    @property
    def target(self) -> str: ...
    @property
    def interval(self) -> int: ...
    @property
    def timeout(self) -> int: ...
    @property
    def retries(self) -> int: ...
    @property
    def start_period(self) -> int: ...
    def __repr__(self) -> str: ...

class ResourceLimits:
    def __init__(
        self,
        *,
        max_memory_bytes: Optional[int] = None,
        max_cpu_percent: Optional[float] = None,
        max_open_files: Optional[int] = None,
    ) -> None: ...
    @property
    def max_memory_bytes(self) -> Optional[int]: ...
    @property
    def max_cpu_percent(self) -> Optional[float]: ...
    @property
    def max_open_files(self) -> Optional[int]: ...
    def __repr__(self) -> str: ...

class LogConfig:
    def __init__(
        self,
        *,
        max_size_bytes: int = 52428800,
        retain_count: int = 5,
        compress_rotated: bool = False,
    ) -> None: ...
    @property
    def max_size_bytes(self) -> int: ...
    @property
    def retain_count(self) -> int: ...
    @property
    def compress_rotated(self) -> bool: ...
    def __repr__(self) -> str: ...

# ---------------------------------------------------------------------------
# Daemon spec builder
# ---------------------------------------------------------------------------

class Daemon:
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
        user: Optional[str] = None,
        health_check: Optional[HealthCheck] = None,
        resource_limits: Optional[ResourceLimits] = None,
        log_config: Optional[LogConfig] = None,
    ) -> None: ...
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
    def user(self) -> Optional[str]: ...
    @property
    def restart_policy(self) -> RestartPolicyType: ...
    @property
    def health_check(self) -> Optional[HealthCheck]: ...
    @property
    def resource_limits(self) -> Optional[ResourceLimits]: ...
    @property
    def log_config(self) -> Optional[LogConfig]: ...
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
    def with_resource_limits(self, limits: ResourceLimits) -> Daemon: ...
    def with_log_config(self, config: LogConfig) -> Daemon: ...

# ---------------------------------------------------------------------------
# Daemon instance (immutable status snapshot)
# ---------------------------------------------------------------------------

class DaemonInstance:
    @property
    def id(self) -> str: ...
    @property
    def name(self) -> str: ...
    @property
    def state(self) -> DaemonStatus: ...
    @property
    def pid(self) -> Optional[int]: ...
    @property
    def started_at(self) -> Optional[str]: ...
    @property
    def stopped_at(self) -> Optional[str]: ...
    @property
    def exit_code(self) -> Optional[int]: ...
    @property
    def restart_count(self) -> int: ...
    @property
    def health(self) -> HealthStatus: ...
    @property
    def stdout_log(self) -> Optional[str]: ...
    @property
    def stderr_log(self) -> Optional[str]: ...
    def __repr__(self) -> str: ...

# ---------------------------------------------------------------------------
# Sync client
# ---------------------------------------------------------------------------

class SyspulseClient:
    def __init__(self, socket_path: Optional[str] = None) -> None: ...
    def __enter__(self) -> SyspulseClient: ...
    def __exit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> bool: ...
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
    def status(self, name: str) -> DaemonInstance: ...
    def list(self) -> List[DaemonInstance]: ...
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

# ---------------------------------------------------------------------------
# Async client
# ---------------------------------------------------------------------------

class AsyncSyspulseClient:
    def __init__(self, socket_path: Optional[str] = None) -> None: ...
    async def __aenter__(self) -> AsyncSyspulseClient: ...
    async def __aexit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> bool: ...
    async def start(
        self,
        name: str,
        *,
        wait: Optional[bool] = None,
        timeout: Optional[int] = None,
    ) -> str: ...
    async def stop(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
        timeout: Optional[int] = None,
    ) -> str: ...
    async def restart(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
        wait: Optional[bool] = None,
    ) -> str: ...
    async def status(self, name: str) -> DaemonInstance: ...
    async def list(self) -> List[DaemonInstance]: ...
    async def logs(
        self,
        name: str,
        *,
        lines: Optional[int] = None,
        stderr: Optional[bool] = None,
    ) -> List[str]: ...
    async def add(self, daemon: Daemon) -> str: ...
    async def remove(
        self,
        name: str,
        *,
        force: Optional[bool] = None,
    ) -> str: ...
    async def is_running(self) -> bool: ...
    async def ping(self) -> bool: ...
    async def shutdown(self) -> str: ...

# ---------------------------------------------------------------------------
# Exceptions
# ---------------------------------------------------------------------------

class SyspulseError(RuntimeError): ...
class DaemonNotFoundError(ValueError): ...
class DaemonAlreadyExistsError(ValueError): ...
class InvalidStateError(RuntimeError): ...

# ---------------------------------------------------------------------------
# Module-level functions
# ---------------------------------------------------------------------------

def from_sys(path: str) -> List[Daemon]: ...
def from_toml(path: str) -> List[Daemon]: ...
def data_dir() -> str: ...
def socket_path() -> str: ...
