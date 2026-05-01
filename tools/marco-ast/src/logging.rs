use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli::OutputMode;
use crate::output::RunPayload;

#[derive(Serialize)]
pub struct LogInputInfo {
    pub source: String,
    pub value: String,
    pub bytes: usize,
    pub sha256: String,
}

#[derive(Serialize)]
pub struct LogOutputInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics_summary: Option<String>,
}

#[derive(Serialize)]
pub struct LogEvent {
    pub timestamp: String,
    pub session_id: String,
    pub mode: OutputMode,
    pub input: LogInputInfo,
    pub output: LogOutputInfo,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LogEvent {
    pub fn build(
        session_id: uuid::Uuid,
        mode: &OutputMode,
        source_kind: &str,
        source_value: &str,
        raw_content: &str,
        payload: &RunPayload,
        error: Option<String>,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(raw_content.as_bytes());
        let digest = hex::encode(hasher.finalize());

        LogEvent {
            timestamp: Utc::now().to_rfc3339(),
            session_id: session_id.to_string(),
            mode: mode.clone(),
            input: LogInputInfo {
                source: source_kind.to_string(),
                value: source_value.to_string(),
                bytes: raw_content.len(),
                sha256: digest,
            },
            output: LogOutputInfo {
                ast: payload.ast.clone(),
                html: payload.html.clone(),
                diagnostics_summary: payload.diagnostics_summary.clone(),
            },
            status: if error.is_some() { "error" } else { "ok" }.to_string(),
            error,
        }
    }
}

pub fn append_log_event(event: &LogEvent, log_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let line = serde_json::to_string(event)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Resolve the default log path relative to the current working directory.
#[allow(dead_code)]
pub fn default_log_path() -> PathBuf {
    PathBuf::from("log.json")
}
