"""Example: using syspulse from Python.

This script demonstrates the full Python API for managing daemons
programmatically. It requires a running syspulse manager instance.
"""

from syspulse import (
    Daemon,
    SyspulseClient,
    HealthCheck,
    HealthCheckType,
    ResourceLimits,
    LogConfig,
    from_sys,
    data_dir,
    socket_path,
)

# ---------------------------------------------------------------------------
# 1. Utility functions — show platform paths
# ---------------------------------------------------------------------------
print(f"Data directory:  {data_dir()}")
print(f"Socket path:     {socket_path()}")

# ---------------------------------------------------------------------------
# 2. Create a daemon programmatically (with typed config objects)
# ---------------------------------------------------------------------------
daemon = Daemon(
    name="my-api",
    command=["python", "-m", "uvicorn", "app:main"],
    working_dir="/app",
    description="My API server",
    tags=["web", "api"],
    env={"PORT": "8080", "LOG_LEVEL": "info"},
    stop_timeout=60,
    health_check=HealthCheck(
        HealthCheckType.Http,
        "http://localhost:8080/health",
        interval=15,
        timeout=5,
        retries=3,
    ),
    resource_limits=ResourceLimits(max_memory_bytes=512 * 1024 * 1024),
    log_config=LogConfig(max_size_bytes=10 * 1024 * 1024, retain_count=3),
)

# Builder methods still work (returns a new Daemon)
daemon = daemon.with_restart_policy("on_failure", max_retries=5)

print(f"\nCreated: {daemon}")
print(f"  name:             {daemon.name}")
print(f"  command:          {daemon.command}")
print(f"  working_dir:      {daemon.working_dir}")
print(f"  restart_policy:   {daemon.restart_policy}")
print(f"  health_check:     {daemon.health_check}")
print(f"  resource_limits:  {daemon.resource_limits}")
print(f"  log_config:       {daemon.log_config}")

# ---------------------------------------------------------------------------
# 3. Load daemons from a .sys config file
# ---------------------------------------------------------------------------
# daemons = from_sys("examples/web-server.sys")
# for d in daemons:
#     print(f"Loaded from config: {d}")

# ---------------------------------------------------------------------------
# 4. Connect to the syspulse manager (context manager)
# ---------------------------------------------------------------------------
with SyspulseClient() as client:
    # Check whether the manager is reachable
    if not client.is_running():
        print("syspulse manager is not running -- start it with `syspulse daemon`")
        raise SystemExit(1)

    # -----------------------------------------------------------------
    # 5. Register and start the daemon
    # -----------------------------------------------------------------
    client.add(daemon)
    client.start("my-api", wait=True, timeout=30)

    # -----------------------------------------------------------------
    # 6. Inspect status (typed DaemonInstance, not a dict)
    # -----------------------------------------------------------------
    status = client.status("my-api")
    print(f"\nStatus of my-api:")
    print(f"  State:         {status.state}")
    print(f"  PID:           {status.pid}")
    print(f"  Restart count: {status.restart_count}")
    print(f"  Health:        {status.health}")
    print(f"  Started at:    {status.started_at}")

    # -----------------------------------------------------------------
    # 7. List all managed daemons
    # -----------------------------------------------------------------
    print("\nAll daemons:")
    for inst in client.list():
        print(f"  {inst.name}: {inst.state}")

    # -----------------------------------------------------------------
    # 8. Read logs
    # -----------------------------------------------------------------
    log_lines = client.logs("my-api", lines=20)
    print(f"\nLast {len(log_lines)} log lines:")
    for line in log_lines:
        print(f"  {line}")

    # -----------------------------------------------------------------
    # 9. Stop and clean up
    # -----------------------------------------------------------------
    client.stop("my-api")
    client.remove("my-api")
    print("\nDaemon stopped and removed.")
