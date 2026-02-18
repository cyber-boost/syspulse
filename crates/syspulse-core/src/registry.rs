use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::daemon::{DaemonInstance, DaemonSpec, HealthStatus};
use crate::error::{Result, SyspulseError};
use crate::lifecycle::LifecycleState;

pub struct Registry {
    conn: Connection,
}

impl Registry {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| SyspulseError::Database(format!("Failed to open database: {}", e)))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| SyspulseError::Database(format!("Failed to set pragmas: {}", e)))?;

        let registry = Self { conn };
        registry.run_migrations()?;
        Ok(registry)
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS daemon_specs (
                    name TEXT PRIMARY KEY,
                    spec_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS daemon_states (
                    name TEXT PRIMARY KEY,
                    instance_id TEXT NOT NULL,
                    state TEXT NOT NULL,
                    pid INTEGER,
                    started_at TEXT,
                    stopped_at TEXT,
                    exit_code INTEGER,
                    restart_count INTEGER DEFAULT 0,
                    health_status TEXT DEFAULT 'unknown',
                    stdout_log TEXT,
                    stderr_log TEXT,
                    FOREIGN KEY (name) REFERENCES daemon_specs(name)
                );",
            )
            .map_err(|e| SyspulseError::Database(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    pub fn register(&self, spec: &DaemonSpec) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let json = serde_json::to_string(spec)?;

        self.conn
            .execute(
                "INSERT INTO daemon_specs (name, spec_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![spec.name, json, now, now],
            )
            .map_err(|e| match e {
                rusqlite::Error::SqliteFailure(err, _)
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    SyspulseError::DaemonAlreadyExists(spec.name.clone())
                }
                _ => SyspulseError::Database(format!("Failed to register daemon: {}", e)),
            })?;

        Ok(())
    }

    pub fn unregister(&self, name: &str) -> Result<()> {
        // Delete state first due to foreign key constraint
        self.conn
            .execute("DELETE FROM daemon_states WHERE name = ?1", params![name])
            .map_err(|e| SyspulseError::Database(format!("Failed to delete state: {}", e)))?;

        let changes = self
            .conn
            .execute("DELETE FROM daemon_specs WHERE name = ?1", params![name])
            .map_err(|e| SyspulseError::Database(format!("Failed to delete spec: {}", e)))?;

        if changes == 0 {
            return Err(SyspulseError::DaemonNotFound(name.to_string()));
        }

        Ok(())
    }

    pub fn get_spec(&self, name: &str) -> Result<DaemonSpec> {
        let json: String = self
            .conn
            .query_row(
                "SELECT spec_json FROM daemon_specs WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    SyspulseError::DaemonNotFound(name.to_string())
                }
                _ => SyspulseError::Database(format!("Failed to get spec: {}", e)),
            })?;

        let spec: DaemonSpec = serde_json::from_str(&json)?;
        Ok(spec)
    }

    pub fn list_specs(&self) -> Result<Vec<DaemonSpec>> {
        let mut stmt = self
            .conn
            .prepare("SELECT spec_json FROM daemon_specs ORDER BY name")
            .map_err(|e| SyspulseError::Database(format!("Failed to prepare query: {}", e)))?;

        let specs = stmt
            .query_map([], |row| {
                let json: String = row.get(0)?;
                Ok(json)
            })
            .map_err(|e| SyspulseError::Database(format!("Failed to list specs: {}", e)))?
            .filter_map(|r| {
                r.ok()
                    .and_then(|json| serde_json::from_str::<DaemonSpec>(&json).ok())
            })
            .collect();

        Ok(specs)
    }

    pub fn update_state(&self, instance: &DaemonInstance) -> Result<()> {
        let state_str = instance.state.to_string();
        let health_str = match instance.health_status {
            HealthStatus::Unknown => "unknown",
            HealthStatus::Healthy => "healthy",
            HealthStatus::Unhealthy => "unhealthy",
            HealthStatus::NotConfigured => "not_configured",
        };
        let started_at = instance.started_at.map(|t| t.to_rfc3339());
        let stopped_at = instance.stopped_at.map(|t| t.to_rfc3339());
        let stdout_log = instance
            .stdout_log
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        let stderr_log = instance
            .stderr_log
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        self.conn
            .execute(
                "INSERT INTO daemon_states (name, instance_id, state, pid, started_at, stopped_at, exit_code, restart_count, health_status, stdout_log, stderr_log)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(name) DO UPDATE SET
                     instance_id = excluded.instance_id,
                     state = excluded.state,
                     pid = excluded.pid,
                     started_at = excluded.started_at,
                     stopped_at = excluded.stopped_at,
                     exit_code = excluded.exit_code,
                     restart_count = excluded.restart_count,
                     health_status = excluded.health_status,
                     stdout_log = excluded.stdout_log,
                     stderr_log = excluded.stderr_log",
                params![
                    instance.spec_name,
                    instance.id,
                    state_str,
                    instance.pid,
                    started_at,
                    stopped_at,
                    instance.exit_code,
                    instance.restart_count,
                    health_str,
                    stdout_log,
                    stderr_log,
                ],
            )
            .map_err(|e| SyspulseError::Database(format!("Failed to update state: {}", e)))?;

        Ok(())
    }

    pub fn get_state(&self, name: &str) -> Result<DaemonInstance> {
        self.conn
            .query_row(
                "SELECT instance_id, state, pid, started_at, stopped_at, exit_code, restart_count, health_status, stdout_log, stderr_log
                 FROM daemon_states WHERE name = ?1",
                params![name],
                |row| {
                    Ok(StateRow {
                        name: name.to_string(),
                        instance_id: row.get(0)?,
                        state: row.get::<_, String>(1)?,
                        pid: row.get(2)?,
                        started_at: row.get::<_, Option<String>>(3)?,
                        stopped_at: row.get::<_, Option<String>>(4)?,
                        exit_code: row.get(5)?,
                        restart_count: row.get::<_, Option<u32>>(6)?,
                        health_status: row.get::<_, Option<String>>(7)?,
                        stdout_log: row.get::<_, Option<String>>(8)?,
                        stderr_log: row.get::<_, Option<String>>(9)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    SyspulseError::DaemonNotFound(name.to_string())
                }
                _ => SyspulseError::Database(format!("Failed to get state: {}", e)),
            })
            .map(|r| r.into_instance())
    }

    pub fn list_states(&self) -> Result<Vec<DaemonInstance>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, instance_id, state, pid, started_at, stopped_at, exit_code, restart_count, health_status, stdout_log, stderr_log
                 FROM daemon_states ORDER BY name",
            )
            .map_err(|e| SyspulseError::Database(format!("Failed to prepare query: {}", e)))?;

        let states = stmt
            .query_map([], |row| {
                Ok(StateRow {
                    name: row.get(0)?,
                    instance_id: row.get(1)?,
                    state: row.get::<_, String>(2)?,
                    pid: row.get(3)?,
                    started_at: row.get::<_, Option<String>>(4)?,
                    stopped_at: row.get::<_, Option<String>>(5)?,
                    exit_code: row.get(6)?,
                    restart_count: row.get::<_, Option<u32>>(7)?,
                    health_status: row.get::<_, Option<String>>(8)?,
                    stdout_log: row.get::<_, Option<String>>(9)?,
                    stderr_log: row.get::<_, Option<String>>(10)?,
                })
            })
            .map_err(|e| SyspulseError::Database(format!("Failed to list states: {}", e)))?
            .filter_map(|r| r.ok())
            .map(|r| r.into_instance())
            .collect();

        Ok(states)
    }
}

struct StateRow {
    name: String,
    instance_id: String,
    state: String,
    pid: Option<u32>,
    started_at: Option<String>,
    stopped_at: Option<String>,
    exit_code: Option<i32>,
    restart_count: Option<u32>,
    health_status: Option<String>,
    stdout_log: Option<String>,
    stderr_log: Option<String>,
}

impl StateRow {
    fn into_instance(self) -> DaemonInstance {
        let state = match self.state.as_str() {
            "stopped" => LifecycleState::Stopped,
            "starting" => LifecycleState::Starting,
            "running" => LifecycleState::Running,
            "stopping" => LifecycleState::Stopping,
            "failed" => LifecycleState::Failed,
            "scheduled" => LifecycleState::Scheduled,
            _ => LifecycleState::Stopped,
        };

        let health_status = match self.health_status.as_deref() {
            Some("healthy") => HealthStatus::Healthy,
            Some("unhealthy") => HealthStatus::Unhealthy,
            Some("not_configured") => HealthStatus::NotConfigured,
            _ => HealthStatus::Unknown,
        };

        let parse_dt = |s: Option<String>| {
            s.and_then(|v| chrono::DateTime::parse_from_rfc3339(&v).ok())
                .map(|dt| dt.with_timezone(&Utc))
        };

        DaemonInstance {
            id: self.instance_id,
            spec_name: self.name,
            state,
            pid: self.pid,
            started_at: parse_dt(self.started_at),
            stopped_at: parse_dt(self.stopped_at),
            exit_code: self.exit_code,
            restart_count: self.restart_count.unwrap_or(0),
            health_status,
            stdout_log: self.stdout_log.map(std::path::PathBuf::from),
            stderr_log: self.stderr_log.map(std::path::PathBuf::from),
        }
    }
}
