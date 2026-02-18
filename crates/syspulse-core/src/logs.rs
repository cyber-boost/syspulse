use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::daemon::LogConfig;
use crate::error::Result;
use crate::paths;

pub struct LogManager {
    data_dir: PathBuf,
}

impl LogManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            data_dir: paths::data_dir(),
        }
    }

    fn log_dir(&self, daemon_name: &str) -> PathBuf {
        self.data_dir.join("logs").join(daemon_name)
    }

    /// Create log directory and return (stdout_path, stderr_path).
    pub fn setup_log_files(&self, daemon_name: &str) -> Result<(PathBuf, PathBuf)> {
        let dir = self.log_dir(daemon_name);
        fs::create_dir_all(&dir)?;

        let stdout_path = dir.join("stdout.log");
        let stderr_path = dir.join("stderr.log");

        Ok((stdout_path, stderr_path))
    }

    /// Rotate log files if they exceed the configured max size.
    /// Rotated files are named with a timestamp suffix. Files beyond retain_count are pruned.
    pub fn rotate_logs(&self, daemon_name: &str, config: &LogConfig) -> Result<()> {
        let dir = self.log_dir(daemon_name);

        for log_name in &["stdout.log", "stderr.log"] {
            let log_path = dir.join(log_name);
            if !log_path.exists() {
                continue;
            }

            let metadata = fs::metadata(&log_path)?;
            if metadata.len() < config.max_size_bytes {
                continue;
            }

            // Rotate: rename current log with timestamp suffix
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let base = log_name.trim_end_matches(".log");
            let rotated_name = format!("{}_{}.log", base, timestamp);
            let rotated_path = dir.join(&rotated_name);

            fs::rename(&log_path, &rotated_path)?;
            tracing::info!(
                daemon = daemon_name,
                file = %log_name,
                rotated_to = %rotated_name,
                "Rotated log file"
            );

            // Prune old rotated files beyond retain_count
            self.prune_rotated(&dir, base, config.retain_count)?;
        }

        Ok(())
    }

    /// Remove oldest rotated log files beyond retain_count.
    fn prune_rotated(&self, dir: &Path, base_name: &str, retain_count: u32) -> Result<()> {
        let prefix = format!("{}_", base_name);

        let mut rotated: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                name.starts_with(&prefix) && name.ends_with(".log")
            })
            .map(|entry| entry.path())
            .collect();

        // Sort by name descending (newest first, since names contain timestamps)
        rotated.sort();
        rotated.reverse();

        for path in rotated.iter().skip(retain_count as usize) {
            tracing::debug!(path = %path.display(), "Pruning old rotated log");
            if let Err(e) = fs::remove_file(path) {
                tracing::warn!(path = %path.display(), error = %e, "Failed to prune log file");
            }
        }

        Ok(())
    }

    /// Read the last N lines from a daemon's log file.
    /// If `stderr` is true, reads from stderr.log; otherwise stdout.log.
    pub fn read_logs(
        &self,
        daemon_name: &str,
        lines: usize,
        stderr: bool,
    ) -> Result<Vec<String>> {
        let dir = self.log_dir(daemon_name);
        let filename = if stderr { "stderr.log" } else { "stdout.log" };
        let log_path = dir.join(filename);

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        tail_file(&log_path, lines)
    }
}

/// Read the last `n` lines from a file efficiently.
fn tail_file(path: &Path, n: usize) -> Result<Vec<String>> {
    if n == 0 {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    if file_size == 0 {
        return Ok(Vec::new());
    }

    // For small files (< 64KB), just read the whole thing
    if file_size < 64 * 1024 {
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        let start = all_lines.len().saturating_sub(n);
        return Ok(all_lines[start..].to_vec());
    }

    // For larger files, read backwards in chunks
    let mut file = file;
    let chunk_size: u64 = 8192;
    let mut remaining = file_size;
    let mut trailing_data = Vec::new();

    loop {
        let read_size = chunk_size.min(remaining);
        remaining -= read_size;

        file.seek(SeekFrom::Start(remaining))?;
        let mut buf = vec![0u8; read_size as usize];
        std::io::Read::read_exact(&mut file, &mut buf)?;

        // Prepend new chunk to trailing data
        buf.extend_from_slice(&trailing_data);
        trailing_data = buf;

        let text = String::from_utf8_lossy(&trailing_data);
        let lines_in_buf: Vec<&str> = text.lines().collect();

        if lines_in_buf.len() > n || remaining == 0 {
            let start = lines_in_buf.len().saturating_sub(n);
            return Ok(lines_in_buf[start..].iter().map(|s| s.to_string()).collect());
        }
    }
}
