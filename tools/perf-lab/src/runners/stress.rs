use crate::adapters;
use crate::cli::StressOptions;
use crate::report;
use crate::runners::{calc_summary, BenchRecord};
use crate::{AppContext, workloads::Workload};

pub fn run(ctx: &AppContext, opts: &StressOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opts.list {
        println!("Stress workloads:");
        for workload in &ctx.workloads {
            println!("- {} ({} bytes, profile: {})", workload.id, workload.bytes, workload.profile);
        }
        return Ok(());
    }

    let engine = opts
        .engine
        .as_deref()
        .unwrap_or(ctx.config.default_engine.as_str());
    let adapter = adapters::get_adapter(engine).map_err(|e| e.to_string())?;

    let selected = select_workloads(&ctx.workloads, opts.workload.as_deref());
    if selected.is_empty() {
        return Err("no workloads selected".into());
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let timestamp_utc = chrono::Utc::now().to_rfc3339();
    let mode_str = format!("{:?}", opts.mode).to_lowercase();
    let mut records: Vec<BenchRecord> = Vec::new();
    let mut total_errors = 0usize;

    for workload in &selected {
        let input_bytes = std::fs::read(&workload.source_path)?;
        let input = String::from_utf8_lossy(&input_bytes);
        let mut samples: Vec<u128> = Vec::with_capacity(opts.loops as usize);
        let mut last_diag = 0usize;
        let mut last_hl = 0usize;
        let mut error_class: Option<String> = None;

        for iter in 0..opts.loops {
            match adapter.run_mode(opts.mode, &input) {
                Ok(run) => {
                    samples.push(run.elapsed_ns);
                    last_diag = run.diagnostics_count;
                    last_hl = run.highlights_count;
                }
                Err(err) => {
                    let ec = classify_error(&err);
                    eprintln!(
                        "  [stress] error on iter {iter} for {}: {err} (class={ec})",
                        workload.id
                    );
                    total_errors += 1;
                    error_class = Some(ec);
                    if !opts.continue_on_error {
                        return Err(format!(
                            "stress run failed for {}: {err}",
                            workload.id
                        )
                        .into());
                    }
                    break;
                }
            }
        }

        let (exit_status, mean_ns, median_ns, p95_ns, stdev_ns, throughput) =
            if error_class.is_some() && samples.is_empty() {
                (String::from("error"), 0u128, 0u128, 0u128, 0u128, 0.0f64)
            } else {
                let (mean, median, p95, stdev) = calc_summary(&samples);
                let tput = if mean == 0 {
                    0.0
                } else {
                    (workload.bytes as f64) / (mean as f64 / 1_000_000_000.0)
                };
                let status = if error_class.is_some() {
                    String::from("partial-error")
                } else {
                    String::from("success")
                };
                (status, mean, median, p95, stdev, tput)
            };

        println!(
            "  stress {} | engine={} mode={} loops={} mean={}ns p95={}ns tput={:.1}MB/s status={}",
            workload.id,
            adapter.id(),
            mode_str,
            samples.len(),
            mean_ns,
            p95_ns,
            throughput / 1_000_000.0,
            exit_status
        );

        records.push(BenchRecord {
            run_id: run_id.clone(),
            timestamp_utc: timestamp_utc.clone(),
            git_sha: ctx.git_sha.clone(),
            engine: adapter.id().to_string(),
            profile: workload.profile.clone(),
            mode: format!("stress-{mode_str}"),
            workload_id: workload.id.clone(),
            workload_bytes: workload.bytes,
            workload_sha256: workload.sha256.clone(),
            iterations: samples.len() as u32,
            mean_ns,
            median_ns,
            p95_ns,
            stdev_ns,
            throughput_bytes_s: throughput,
            exit_status,
            error_class,
            diagnostics_count: last_diag,
            highlights_count: last_hl,
        });
    }

    if total_errors > 0 {
        eprintln!("stress completed with {total_errors} error(s)");
    }

    report::persist_bench_records(ctx, "stress", &records)?;
    println!("stress completed: {} record(s)", records.len());
    Ok(())
}

fn classify_error(msg: &str) -> String {
    let lower = msg.to_lowercase();
    if lower.contains("unsupported") {
        String::from("unsupported-mode")
    } else if lower.contains("panic") || lower.contains("thread") {
        String::from("panic")
    } else if lower.contains("timeout") || lower.contains("timed out") {
        String::from("timeout")
    } else if lower.contains("oom") || lower.contains("out of memory") || lower.contains("alloc") {
        String::from("oom")
    } else {
        String::from("engine-error")
    }
}

fn select_workloads<'a>(workloads: &'a [Workload], workload_id: Option<&str>) -> Vec<&'a Workload> {
    workloads
        .iter()
        .filter(|w| match workload_id {
            Some(id) => w.id == id,
            None => true,
        })
        .collect()
}
