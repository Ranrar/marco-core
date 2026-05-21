use crate::adapters;
use crate::cli::{BenchOptions, Mode};
use crate::report;
use crate::runners::{calc_summary, BenchRecord};
use crate::{workloads, AppContext, workloads::Workload};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub fn run(ctx: &AppContext, opts: &BenchOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opts.list {
        list_catalog(ctx);
        return Ok(());
    }

    if opts.criterion && opts.criterion_sample_size < 10 {
        return Err("--criterion-sample-size must be >= 10".into());
    }

    let baseline_manifest = opts
        .baseline_manifest
        .clone()
        .or_else(|| ctx.config.baseline_manifest.clone());

    if opts.check_manifest_drift && baseline_manifest.is_none() {
        return Err("--check-manifest-drift requires --baseline-manifest or config.baseline_manifest"
            .into());
    }

    if let Some(path) = baseline_manifest {
        if path.exists() {
            let expected = workloads::load_manifest(&path)?;
            let drift = workloads::compute_manifest_drift(&expected, &ctx.workloads);
            if !drift.is_empty() {
                println!("manifest drift detected: {}", path.display());
                for id in &drift.missing_ids {
                    println!("- missing: {id}");
                }
                for id in &drift.unexpected_ids {
                    println!("- unexpected: {id}");
                }
                for id in &drift.changed_ids {
                    println!("- changed: {id}");
                }

                if opts.check_manifest_drift || ctx.config.strict {
                    return Err("workload manifest drift detected".into());
                }
            } else {
                println!("manifest check ok: {}", path.display());
            }
        } else if opts.check_manifest_drift || ctx.config.strict {
            return Err(format!("baseline manifest not found: {}", path.display()).into());
        } else {
            println!(
                "warning: baseline manifest configured but missing: {}",
                path.display()
            );
        }
    }

    let engine = opts
        .engine
        .as_deref()
        .unwrap_or(ctx.config.default_engine.as_str());
    let adapter = if opts.criterion {
        None
    } else {
        Some(adapters::get_adapter(engine).map_err(|e| e.to_string())?)
    };

    let selected = select_workloads(&ctx.workloads, opts.workload.as_deref());
    if selected.is_empty() {
        return Err("no workloads selected".into());
    }

    if let Some(manifest_path) = &opts.manifest_out {
        let selected_manifest: Vec<Workload> = selected.iter().map(|w| (*w).clone()).collect();
        workloads::write_manifest(manifest_path, &selected_manifest)?;
        println!("workload manifest: {}", manifest_path.display());
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let timestamp_utc = chrono::Utc::now().to_rfc3339();
    let mut records = Vec::new();

    for workload in selected {
        if opts.criterion {
            let criterion = run_criterion(ctx, engine, opts, workload)?;
            let throughput_bytes_s = if criterion.mean_ns == 0 {
                0.0
            } else {
                (workload.bytes as f64) / (criterion.mean_ns as f64 / 1_000_000_000.0)
            };

            records.push(BenchRecord {
                run_id: run_id.clone(),
                timestamp_utc: timestamp_utc.clone(),
                git_sha: ctx.git_sha.clone(),
                engine: engine.to_string(),
                profile: workload.profile.clone(),
                mode: format!("{:?}", opts.mode).to_lowercase(),
                workload_id: workload.id.clone(),
                workload_bytes: workload.bytes,
                workload_sha256: workload.sha256.clone(),
                iterations: opts.criterion_sample_size as u32,
                mean_ns: criterion.mean_ns,
                median_ns: criterion.median_ns,
                p95_ns: criterion.upper_ci_ns,
                stdev_ns: criterion.std_dev_ns,
                throughput_bytes_s,
                exit_status: String::from("success"),
                error_class: None,
                diagnostics_count: 0,
                highlights_count: 0,
            });
        } else {
            let adapter = adapter
                .as_ref()
                .ok_or("internal error: adapter missing in non-criterion path")?;
            let input_bytes = std::fs::read(&workload.source_path)?;
            let input = String::from_utf8_lossy(&input_bytes);

            let mut samples = Vec::new();
            let mut diagnostics_count = 0usize;
            let mut highlights_count = 0usize;

            for _ in 0..opts.iterations {
                let run = adapter
                    .run_mode(opts.mode, &input)
                    .map_err(|e| format!("engine run failed for {}: {}", workload.id, e))?;
                samples.push(run.elapsed_ns);
                diagnostics_count = run.diagnostics_count;
                highlights_count = run.highlights_count;
            }

            let (mean_ns, median_ns, p95_ns, stdev_ns) = calc_summary(&samples);
            let throughput_bytes_s = if mean_ns == 0 {
                0.0
            } else {
                (workload.bytes as f64) / (mean_ns as f64 / 1_000_000_000.0)
            };

            records.push(BenchRecord {
                run_id: run_id.clone(),
                timestamp_utc: timestamp_utc.clone(),
                git_sha: ctx.git_sha.clone(),
                engine: adapter.id().to_string(),
                profile: workload.profile.clone(),
                mode: format!("{:?}", opts.mode).to_lowercase(),
                workload_id: workload.id.clone(),
                workload_bytes: workload.bytes,
                workload_sha256: workload.sha256.clone(),
                iterations: opts.iterations,
                mean_ns,
                median_ns,
                p95_ns,
                stdev_ns,
                throughput_bytes_s,
                exit_status: String::from("success"),
                error_class: None,
                diagnostics_count,
                highlights_count,
            });
        }
    }

    report::persist_bench_records(ctx, "bench", &records)?;
    println!("bench completed: {} record(s)", records.len());
    Ok(())
}

fn list_catalog(ctx: &AppContext) {
    println!("Engines:");
    for engine in adapters::descriptors() {
        println!(
            "- {} (implemented: {}, {})",
            engine.id, engine.implemented, engine.notes
        );
    }
    println!("\nWorkloads:");
    for workload in &ctx.workloads {
        println!(
            "- {} [{}] ({} bytes)",
            workload.id, workload.profile, workload.bytes
        );
    }
}

fn select_workloads<'a>(
    workloads: &'a [Workload],
    workload_id: Option<&str>,
) -> Vec<&'a Workload> {
    workloads
        .iter()
        .filter(|w| match workload_id {
            Some(id) => w.id == id,
            None => true,
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct CriterionEstimates {
    mean: CriterionStat,
    median: CriterionStat,
    std_dev: CriterionStat,
}

#[derive(Debug, Deserialize)]
struct CriterionStat {
    confidence_interval: CriterionConfidenceInterval,
    point_estimate: f64,
}

#[derive(Debug, Deserialize)]
struct CriterionConfidenceInterval {
    upper_bound: f64,
}

#[derive(Debug)]
struct CriterionResult {
    mean_ns: u128,
    median_ns: u128,
    std_dev_ns: u128,
    upper_ci_ns: u128,
}

fn run_criterion(
    ctx: &AppContext,
    engine: &str,
    opts: &BenchOptions,
    workload: &Workload,
) -> Result<CriterionResult, Box<dyn std::error::Error>> {
    if engine != "marco-core" {
        return Err("criterion backend currently supports only marco-core".into());
    }

    let bench_id = sanitize_bench_id(engine, opts.mode, &workload.id);
    let warmup_secs = (opts.criterion_warmup_ms as f64) / 1000.0;
    let measurement_secs = (opts.criterion_measurement_ms as f64) / 1000.0;

    let output = std::process::Command::new("cargo")
        .arg("bench")
        .arg("--manifest-path")
        .arg("tools/perf-lab/Cargo.toml")
        .arg("--bench")
        .arg("marco_core_modes")
        .arg("--")
        .arg("--noplot")
        .arg("--sample-size")
        .arg(opts.criterion_sample_size.to_string())
        .arg("--warm-up-time")
        .arg(format!("{warmup_secs:.3}"))
        .arg("--measurement-time")
        .arg(format!("{measurement_secs:.3}"))
        .env("PERF_LAB_ENGINE", engine)
        .env("PERF_LAB_MODE", format!("{:?}", opts.mode).to_lowercase())
        .env("PERF_LAB_WORKLOAD_PATH", workload.source_path.as_os_str())
        .env("PERF_LAB_BENCH_ID", &bench_id)
        .env(
            "PERF_LAB_CRITERION_WARMUP_MS",
            opts.criterion_warmup_ms.to_string(),
        )
        .env(
            "PERF_LAB_CRITERION_MEASUREMENT_MS",
            opts.criterion_measurement_ms.to_string(),
        )
        .current_dir(&ctx.repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("criterion execution failed: {stderr}").into());
    }

    let estimates_path = criterion_estimates_path(&ctx.repo_root, &bench_id);
    let content = std::fs::read_to_string(&estimates_path)?;
    let estimates: CriterionEstimates = serde_json::from_str(&content)?;

    Ok(CriterionResult {
        mean_ns: estimates.mean.point_estimate.max(0.0).round() as u128,
        median_ns: estimates.median.point_estimate.max(0.0).round() as u128,
        std_dev_ns: estimates.std_dev.point_estimate.max(0.0).round() as u128,
        upper_ci_ns: estimates
            .mean
            .confidence_interval
            .upper_bound
            .max(0.0)
            .round() as u128,
    })
}

fn criterion_estimates_path(repo_root: &Path, bench_id: &str) -> PathBuf {
    repo_root
        .join("tools/perf-lab/target/criterion")
        .join(bench_id)
        .join("new")
        .join("estimates.json")
}

fn sanitize_bench_id(engine: &str, mode: Mode, workload_id: &str) -> String {
    let base = format!("{engine}_{:?}_{workload_id}", mode);
    let lower = base.to_lowercase();
    lower
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' | '-' | '_' => ch,
            _ => '_',
        })
        .collect()
}
