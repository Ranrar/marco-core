use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "perf-lab", version, about = "Performance and stress tooling for marco-core")]
pub struct Args {
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Enable strict mode for runner exits")]
    pub strict: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Bench(BenchOptions),
    Stress(StressOptions),
    Compare(CompareOptions),
    Report(ReportOptions),
    Regression(RegressionOptions),
}

#[derive(Debug, clap::Args)]
pub struct BenchOptions {
    #[arg(long)]
    pub list: bool,

    #[arg(long, value_name = "ENGINE")]
    pub engine: Option<String>,

    #[arg(long, value_name = "WORKLOAD_ID")]
    pub workload: Option<String>,

    #[arg(long, value_enum, default_value_t = Mode::E2e)]
    pub mode: Mode,

    #[arg(long, default_value_t = 1)]
    pub iterations: u32,

    #[arg(long, help = "Use Criterion backend instead of direct iteration timing")]
    pub criterion: bool,

    #[arg(long, default_value_t = 10, value_name = "N")]
    pub criterion_sample_size: usize,

    #[arg(long, default_value_t = 200, value_name = "MS")]
    pub criterion_warmup_ms: u64,

    #[arg(long, default_value_t = 500, value_name = "MS")]
    pub criterion_measurement_ms: u64,

    #[arg(long, value_name = "PATH")]
    pub manifest_out: Option<PathBuf>,

    #[arg(long, value_name = "PATH")]
    pub baseline_manifest: Option<PathBuf>,

    #[arg(long, help = "Check discovered workloads against baseline manifest and fail on drift")]
    pub check_manifest_drift: bool,
}

#[derive(Debug, clap::Args)]
pub struct StressOptions {
    #[arg(long)]
    pub list: bool,

    #[arg(long, value_name = "ENGINE")]
    pub engine: Option<String>,

    #[arg(long, value_name = "WORKLOAD_ID")]
    pub workload: Option<String>,

    #[arg(long, value_enum, default_value_t = Mode::E2e)]
    pub mode: Mode,

    #[arg(long, default_value_t = 10)]
    pub loops: u32,

    #[arg(long, help = "Continue on error instead of aborting")]
    pub continue_on_error: bool,
}

#[derive(Debug, clap::Args)]
pub struct CompareOptions {
    #[arg(long)]
    pub list: bool,

    #[arg(long = "engine", value_name = "ENGINE", action = clap::ArgAction::Append)]
    pub engines: Vec<String>,

    #[arg(long, value_name = "WORKLOAD_ID")]
    pub workload: Option<String>,

    #[arg(long, value_enum, default_value_t = Mode::E2e)]
    pub mode: Mode,

    #[arg(long, default_value_t = 10)]
    pub iterations: u32,
}

#[derive(Debug, clap::Args)]
pub struct ReportOptions {
    #[arg(long, value_name = "PATH", help = "Input perf-lab JSON artifact to re-render")]
    pub input: Option<PathBuf>,

    #[arg(long, value_name = "PATH", help = "Input hyperfine JSON export to ingest and normalize")]
    pub hyperfine_input: Option<PathBuf>,

    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
    pub format: ReportFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Mode {
    Parse,
    Render,
    E2e,
    Intelligence,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReportFormat {
    Json,
    Csv,
    Markdown,
}

#[derive(Debug, clap::Args)]
pub struct RegressionOptions {
    #[arg(long, value_name = "PATH", help = "Baseline perf-lab JSON artifact to compare against")]
    pub baseline: PathBuf,

    #[arg(long, value_name = "PATH", help = "Current run perf-lab JSON artifact to check")]
    pub current: PathBuf,

    #[arg(
        long,
        default_value_t = 10.0,
        value_name = "PCT",
        help = "Warn when median regresses by more than this percent (default: 10)"
    )]
    pub warn_threshold: f64,

    #[arg(
        long,
        default_value_t = 20.0,
        value_name = "PCT",
        help = "Fail (exit 1) when median regresses by more than this percent (default: 20)"
    )]
    pub fail_threshold: f64,

    #[arg(
        long,
        help = "Only gate on workloads tagged as critical (contain 'critical' in profile or id, \
                or are spec:commonmark / fixture:pathological:*)"
    )]
    pub critical_only: bool,

    #[arg(
        long,
        default_value_t = 2,
        value_name = "N",
        help = "Minimum number of regressions required to trigger a hard failure (default: 2)"
    )]
    pub min_failures: usize,

    #[arg(
        long,
        value_enum,
        value_name = "MODE",
        help = "Only compare records in this mode (parse/render/e2e/intelligence); \
                default compares every mode present in both files"
    )]
    pub mode: Option<Mode>,
}
