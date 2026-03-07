use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing_subscriber::{fmt, EnvFilter};

const DEFAULT_REPORT_DIR: &str = "reports/sessions";

#[derive(Debug, Clone)]
pub struct SessionReporter {
    session_id: String,
    report_path: Option<PathBuf>,
}

impl SessionReporter {
    fn disabled() -> Self {
        Self {
            session_id: "disabled".to_string(),
            report_path: None,
        }
    }

    pub fn log_session_start(&self, mode: &str, config_path: &str, video_path: Option<&str>) {
        if let Some(report_path) = &self.report_path {
            tracing::info!(
                "SESSION_REPORT start id={} mode={} config={} video={} file={}",
                self.session_id,
                mode,
                config_path,
                video_path.unwrap_or("-"),
                report_path.display()
            );
        }
    }

    pub fn log_session_finish(&self, status: &str) {
        if self.report_path.is_some() {
            tracing::info!(
                "SESSION_REPORT end id={} status={}",
                self.session_id,
                status
            );
        }
    }

    pub fn log_session_failure(&self, error: &anyhow::Error) {
        if self.report_path.is_some() {
            tracing::error!(
                "SESSION_REPORT end id={} status=error error={}",
                self.session_id,
                error
            );
        }
    }
}

pub fn init_logging(
    verbose: bool,
    debug: bool,
    report: bool,
    report_file: Option<&str>,
) -> Result<SessionReporter> {
    if report || report_file.is_some() {
        init_file_logging(verbose, debug, report_file)
    } else {
        init_sink_logging();
        Ok(SessionReporter::disabled())
    }
}

fn init_file_logging(
    verbose: bool,
    debug: bool,
    report_file: Option<&str>,
) -> Result<SessionReporter> {
    let session_id = build_session_id();
    let report_path = report_file
        .map(PathBuf::from)
        .unwrap_or_else(|| default_report_path(&session_id));

    if let Some(parent) = report_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create report directory {}", parent.display()))?;
    }

    let mut bootstrap_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&report_path)
        .with_context(|| format!("Failed to open session report {}", report_path.display()))?;
    writeln!(
        bootstrap_file,
        "\n=== SESSION_REPORT bootstrap id={} pid={} ===",
        session_id,
        process::id()
    )
    .with_context(|| {
        format!(
            "Failed to write bootstrap record to {}",
            report_path.display()
        )
    })?;

    let writer_file = bootstrap_file
        .try_clone()
        .with_context(|| format!("Failed to clone report file {}", report_path.display()))?;

    let filter = if debug {
        EnvFilter::new("trace")
    } else if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("debug")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(fmt::time::uptime())
        .with_writer(move || {
            writer_file
                .try_clone()
                .expect("failed to clone session report file")
        })
        .with_ansi(false)
        .with_level(true)
        .init();

    tracing::info!(
        "Session report enabled: id={} path={}",
        session_id,
        report_path.display()
    );

    Ok(SessionReporter {
        session_id,
        report_path: Some(report_path),
    })
}

fn init_sink_logging() {
    fmt()
        .with_env_filter(EnvFilter::new("off"))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(fmt::time::uptime())
        .with_writer(std::io::sink)
        .with_ansi(false)
        .with_level(true)
        .init();
}

fn build_session_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{}-{:03}-{}",
        now.as_secs(),
        now.subsec_millis(),
        process::id()
    )
}

fn default_report_path(session_id: &str) -> PathBuf {
    PathBuf::from(DEFAULT_REPORT_DIR).join(format!("session-{session_id}.log"))
}

#[cfg(test)]
mod tests {
    use super::{build_session_id, default_report_path};
    use std::path::PathBuf;
    use std::process;

    #[test]
    fn default_report_path_uses_sessions_directory() {
        assert_eq!(
            default_report_path("abc123"),
            PathBuf::from("reports/sessions/session-abc123.log")
        );
    }

    #[test]
    fn session_id_contains_current_pid() {
        let id = build_session_id();
        assert!(id.ends_with(&format!("-{}", process::id())));
    }
}
