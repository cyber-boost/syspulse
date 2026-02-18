"""Example: using syspulse from Python.

This script demonstrates the full Python API for managing daemons
programmatically. It requires a running syspulse manager instance.
"""

from syspulse import Daemon, SyspulseClient, from_toml

# ---------------------------------------------------------------------------
# 1. Create a daemon programmatically
# ---------------------------------------------------------------------------
daemon = Daemon(
    name="my-api",
    command=["python", "-m", "uvicorn", "app:main"],
    working_dir="/app",
    description="My API server",
    tags=["web", "api"],
    env={"PORT": "8080", "LOG_LEVEL": "info"},
    stop_timeout=60,
)

# Add a health check (returns a new Daemon -- immutable builder pattern)
daemon = daemon.with_health_check(
    "http",
    "http://localhost:8080/health",
    interval=15,
    timeout=5,
    retries=3,
)

# Set a restart policy
daemon = daemon.with_restart_policy("on_failure", max_retries=5)

print(f"Created: {daemon}")
print(f"  name:           {daemon.name}")
print(f"  command:        {daemon.command}")
print(f"  working_dir:    {daemon.working_dir}")
print(f"  restart_policy: {daemon.restart_policy}")

# ---------------------------------------------------------------------------
# 2. Load daemons from a TOML config file
# ---------------------------------------------------------------------------
# daemons = from_toml("examples/web-server.toml")
# for d in daemons:
#     print(f"Loaded from config: {d}")

# ---------------------------------------------------------------------------
# 3. Connect to the syspulse manager
# ---------------------------------------------------------------------------
client = SyspulseClient()  # uses default socket path

# Check whether the manager is reachable
if not client.is_running():
    print("syspulse manager is not running -- start it with `syspulse daemon`")
    raise SystemExit(1)

# ---------------------------------------------------------------------------
# 4. Register and start the daemon
# ---------------------------------------------------------------------------
client.add(daemon)
client.start("my-api", wait=True, timeout=30)

# ---------------------------------------------------------------------------
# 5. Inspect status
# ---------------------------------------------------------------------------
status = client.status("my-api")
print(f"\nStatus of my-api:")
print(f"  State:         {status['state']}")
print(f"  PID:           {status['pid']}")
print(f"  Restart count: {status['restart_count']}")
print(f"  Health:        {status['health']}")

# ---------------------------------------------------------------------------
# 6. List all managed daemons
# ---------------------------------------------------------------------------
print("\nAll daemons:")
for d in client.list():
    print(f"  {d['name']}: {d['state']}")

# ---------------------------------------------------------------------------
# 7. Read logs
# ---------------------------------------------------------------------------
log_lines = client.logs("my-api", lines=20)
print(f"\nLast {len(log_lines)} log lines:")
for line in log_lines:
    print(f"  {line}")

# ---------------------------------------------------------------------------
# 8. Stop and clean up
# ---------------------------------------------------------------------------
client.stop("my-api")
client.remove("my-api")
print("\nDaemon stopped and removed.")
