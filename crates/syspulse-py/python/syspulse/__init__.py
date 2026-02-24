"""syspulse -- Cross-platform daemon manager."""

from syspulse._syspulse import (
    __version__,
    # Core classes
    Daemon,
    DaemonInstance,
    SyspulseClient,
    # Config classes
    HealthCheck,
    ResourceLimits,
    LogConfig,
    # Enums
    DaemonStatus,
    HealthStatus,
    HealthCheckType,
    RestartPolicyType,
    # Exceptions
    SyspulseError,
    DaemonNotFoundError,
    DaemonAlreadyExistsError,
    InvalidStateError,
    # Functions
    from_sys,
    from_toml,
    data_dir,
    socket_path,
)

from syspulse.async_client import AsyncSyspulseClient

__all__ = [
    # Version
    "__version__",
    # Core classes
    "Daemon",
    "DaemonInstance",
    "SyspulseClient",
    "AsyncSyspulseClient",
    # Config classes
    "HealthCheck",
    "ResourceLimits",
    "LogConfig",
    # Enums
    "DaemonStatus",
    "HealthStatus",
    "HealthCheckType",
    "RestartPolicyType",
    # Exceptions
    "SyspulseError",
    "DaemonNotFoundError",
    "DaemonAlreadyExistsError",
    "InvalidStateError",
    # Functions
    "from_sys",
    "from_toml",
    "data_dir",
    "socket_path",
]
