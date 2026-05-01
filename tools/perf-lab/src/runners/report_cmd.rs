use crate::cli::{ReportFormat, ReportOptions};
use crate::report;
use crate::runners::BenchRecord;
use crate::AppContext;
use serde::Deserialize;

pub fn run(_ctx: &AppContext, opts: &ReportOptions) -> Result<(), Box<dyn std::error::Error>> {
    // ── hyperfine ingestion path ──────────────────────────────────────────────
    if let Some(hf_input) = &opts.hyperfine_input {
        let records = ingest_hyperfine(hf_input)?;
        return emit(opts, &records);
    }

    // ── standard perf-lab JSON re-render ─────────────────────────────────────
    let input = opts
        .input
        .as_ref()
        .ok_or("--input or --hyperfine-input is required for the report command")?;

    let content = std::fs::read_to_string(input)?;
    let records: Vec<BenchRecord> = serde_json::from_str(&content)?;
    emit(opts, &records)
}

fn emit(opts: &ReportOptions, records: &[BenchRecord]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(output) = &opts.output {
        match opts.format {
            ReportFormat::Json => report::json::write(output, records)?,
            ReportFormat::Csv => report::csv::write_bench_records(output, records)?,
            ReportFormat::Markdown => report::markdown::write_bench_summary(output, records)?,
        }
        println!("report generated: {}", output.display());
        return Ok(());
    }

    match opts.format {
        ReportFormat::Json => println!("{}", serde_json::to_string_pretty(records)?),
        ReportFormat::Csv => println!("{}", report::csv::to_string(records)),
        ReportFormat::Markdown => println!("{}", report::markdown::to_string(records)),
    }
    Ok(())
}

// ── Hyperfine JSON schema (subset we need) ────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HyperfineReport {
    results: Vec<HyperfineResult>,
}

#[derive(Debug, Deserialize)]
struct HyperfineResult {
    command: String,
    /// Wall-clock mean in seconds
    mean: f64,
    stddev: Option<f64>,
    median: f64,
    #[allow(dead_code)]
    min: f64,
    max: f64,
    times: Option<Vec<f64>>,
}

/// Convert a hyperfine JSON export into normalized `BenchRecord`s.
///
/// Engine and workload names are extracted from the command string by looking
/// for `--engine <name>` and `--workload <id>` tokens. Falls back to the full
/// command as the workload_id when they cannot be parsed.
fn ingest_hyperfine(path: &std::path::Path) -> Result<Vec<BenchRecord>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let report: HyperfineReport = serde_json::from_str(&content)?;

    let run_id = uuid::Uuid::new_v4().to_string();
    let timestamp_utc = chrono::Utc::now().to_rfc3339();

    let mut records = Vec::new();
    for result in &report.results {
        let engine = extract_flag(&result.command, "--engine")
            .unwrap_or_else(|| String::from("unknown"));
        let workload_id = extract_flag(&result.command, "--workload")
            .unwrap_or_else(|| result.command.clone());
        let mode = extract_flag(&result.command, "--mode")
            .unwrap_or_else(|| String::from("e2e"));

        let mean_ns = (result.mean * 1_000_000_000.0) as u128;
        let median_ns = (result.median * 1_000_000_000.0) as u128;
        let stdev_ns = result
            .stddev
            .map(|s| (s * 1_000_000_000.0) as u128)
            .unwrap_or(0);

        // p95 from raw times if available, otherwise fall back to max
        let p95_ns = if let Some(times) = &result.times {
            let mut sorted: Vec<u128> = times.iter().map(|t| (*t * 1_000_000_000.0) as u128).collect();
            sorted.sort_unstable();
            let idx = ((sorted.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
            sorted[idx.min(sorted.len().saturating_sub(1))]
        } else {
            (result.max * 1_000_000_000.0) as u128
        };

        let iterations = result.times.as_ref().map(|t| t.len() as u32).unwrap_or(0);

        records.push(BenchRecord {
            run_id: run_id.clone(),
            timestamp_utc: timestamp_utc.clone(),
            git_sha: String::from("unknown"),
            engine,
            profile: String::from("hyperfine"),
            mode: format!("hyperfine-{mode}"),
            workload_id,
            workload_bytes: 0,
            workload_sha256: String::new(),
            iterations,
            mean_ns,
            median_ns,
            p95_ns,
            stdev_ns,
            throughput_bytes_s: 0.0,
            exit_status: String::from("success"),
            error_class: None,
            diagnostics_count: 0,
            highlights_count: 0,
        });
    }

    println!("hyperfine: ingested {} result(s) from {}", records.len(), path.display());
    Ok(records)
}

/// Extract the value of a CLI flag like `--engine marco-core` from a command string.
fn extract_flag(command: &str, flag: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    tokens
        .windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].trim_matches('\'').to_string())
}

