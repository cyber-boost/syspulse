use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SYSPULSE_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(unix)]
    {
        dirs::home_dir().unwrap_or_default().join(".syspulse")
    }
    #[cfg(windows)]
    {
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("syspulse")
    }
}

pub fn db_path() -> PathBuf {
    data_dir().join("syspulse.db")
}

pub fn logs_dir() -> PathBuf {
    data_dir().join("logs")
}

pub fn daemon_log_dir(name: &str) -> PathBuf {
    logs_dir().join(name)
}

pub fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        data_dir().join("syspulse.sock")
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\syspulse")
    }
}

pub fn pid_path() -> PathBuf {
    data_dir().join("syspulse.pid")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir())?;
    std::fs::create_dir_all(logs_dir())?;
    Ok(())
}
