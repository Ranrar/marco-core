mod adapters;
mod cli;
mod config;
mod report;
mod runners;
mod workloads;

use clap::Parser;
use cli::{Args, Command};
use config::PerfLabConfig;
use std::path::{Path, PathBuf};
use workloads::Workload;

pub struct AppContext {
    pub repo_root: PathBuf,
    pub config: PerfLabConfig,
    pub workloads: Vec<Workload>,
    pub git_sha: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let repo_root = detect_repo_root()?;
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(default_config_path);
    let mut config = PerfLabConfig::load(&repo_root, &config_path)?;
    if args.strict {
        config.strict = true;
    }

    let workloads = workloads::discover_workloads(&repo_root, &config)?;
    let git_sha = detect_git_sha(&repo_root).unwrap_or_else(|_| String::from("unknown"));

    let ctx = AppContext {
        repo_root,
        config,
        workloads,
        git_sha,
    };

    match args.command {
        Command::Bench(opts) => runners::benchmark::run(&ctx, &opts),
        Command::Stress(opts) => runners::stress::run(&ctx, &opts),
        Command::Compare(opts) => runners::compare::run(&ctx, &opts),
        Command::Report(opts) => runners::report_cmd::run(&ctx, &opts),
        Command::Regression(opts) => runners::regression::run(&ctx, &opts),
    }
}

fn default_config_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("perf-lab.ron")
}

fn detect_repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let canonical = root.canonicalize()?;
    Ok(canonical)
}

fn detect_git_sha(repo_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()?;

    if !output.status.success() {
        return Err("git rev-parse failed".into());
    }

    let sha = String::from_utf8(output.stdout)?;
    Ok(sha.trim().to_string())
}
