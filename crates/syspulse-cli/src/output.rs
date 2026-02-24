use chrono::Utc;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use owo_colors::OwoColorize;
use syspulse_core::daemon::{DaemonInstance, HealthStatus};
use syspulse_core::lifecycle::LifecycleState;

use crate::commands::OutputFormat;

fn colors_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
}

pub fn format_instance(instance: &DaemonInstance, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(instance).unwrap_or_default(),
        OutputFormat::Table => format_instance_detail(instance),
    }
}

pub fn format_instance_list(instances: &[DaemonInstance], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(instances).unwrap_or_default(),
        OutputFormat::Table => format_table(instances),
    }
}

fn format_table(instances: &[DaemonInstance]) -> String {
    if instances.is_empty() {
        return "No daemons configured.".to_string();
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Name", "State", "PID", "Uptime", "Health", "Restarts"]);

    for inst in instances {
        let state_str = colorize_state(&inst.state);
        let pid = inst
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".into());
        let uptime = format_uptime(inst);
        let health = format_health(&inst.health_status);
        table.add_row(vec![
            &inst.spec_name,
            &state_str,
            &pid,
            &uptime,
            &health,
            &inst.restart_count.to_string(),
        ]);
    }
    table.to_string()
}

fn colorize_state(state: &LifecycleState) -> String {
    let label = state.to_string();
    if !colors_enabled() {
        return label;
    }
    match state {
        LifecycleState::Running => label.green().to_string(),
        LifecycleState::Stopped => label.dimmed().to_string(),
        LifecycleState::Failed => label.red().to_string(),
        LifecycleState::Starting => label.yellow().to_string(),
        LifecycleState::Stopping => label.yellow().to_string(),
        LifecycleState::Scheduled => label.cyan().to_string(),
    }
}

fn format_health(status: &HealthStatus) -> String {
    let label = match status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Unhealthy => "unhealthy",
        HealthStatus::Unknown => "unknown",
        HealthStatus::NotConfigured => "-",
    };
    if !colors_enabled() {
        return label.to_string();
    }
    match status {
        HealthStatus::Healthy => label.green().to_string(),
        HealthStatus::Unhealthy => label.red().to_string(),
        HealthStatus::Unknown => label.dimmed().to_string(),
        HealthStatus::NotConfigured => label.dimmed().to_string(),
    }
}

fn format_uptime(instance: &DaemonInstance) -> String {
    let started = match instance.started_at {
        Some(t) => t,
        None => return "-".to_string(),
    };

    if !instance.state.is_active() {
        return "-".to_string();
    }

    let duration = Utc::now().signed_duration_since(started);
    let total_secs = duration.num_seconds();
    if total_secs < 0 {
        return "-".to_string();
    }

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

fn format_instance_detail(instance: &DaemonInstance) -> String {
    let mut lines = Vec::new();

    let name_display = if colors_enabled() {
        instance.spec_name.bold().to_string()
    } else {
        instance.spec_name.clone()
    };

    lines.push(format!("Name:       {}", name_display));
    lines.push(format!("ID:         {}", instance.id));
    lines.push(format!("State:      {}", colorize_state(&instance.state)));
    lines.push(format!(
        "PID:        {}",
        instance
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!(
        "Health:     {}",
        format_health(&instance.health_status)
    ));
    lines.push(format!("Restarts:   {}", instance.restart_count));
    lines.push(format!("Uptime:     {}", format_uptime(instance)));

    if let Some(ref t) = instance.started_at {
        lines.push(format!("Started:    {}", t.format("%Y-%m-%d %H:%M:%S UTC")));
    }
    if let Some(ref t) = instance.stopped_at {
        lines.push(format!("Stopped:    {}", t.format("%Y-%m-%d %H:%M:%S UTC")));
    }
    if let Some(code) = instance.exit_code {
        lines.push(format!("Exit Code:  {}", code));
    }
    if let Some(ref p) = instance.stdout_log {
        lines.push(format!("Stdout Log: {}", p.display()));
    }
    if let Some(ref p) = instance.stderr_log {
        lines.push(format!("Stderr Log: {}", p.display()));
    }

    lines.join("\n")
}
