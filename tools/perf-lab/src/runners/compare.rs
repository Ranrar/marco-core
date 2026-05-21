use crate::adapters;
use crate::cli::CompareOptions;
use crate::report;
use crate::runners::{calc_summary, BenchRecord};
use crate::{AppContext, workloads::Workload};

pub fn run(ctx: &AppContext, opts: &CompareOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opts.list {
        println!("Available engines:");
        for engine in adapters::descriptors() {
            let status = if engine.implemented { "ready" } else { "stub" };
            println!("- {} [{}] — {}", engine.id, status, engine.notes);
        }
        return Ok(());
    }

    if opts.engines.len() < 2 {
        return Err("compare requires at least two --engine values".into());
    }

    let selected = select_workloads(&ctx.workloads, opts.workload.as_deref());
    if selected.is_empty() {
        return Err("no workloads selected".into());
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let timestamp_utc = chrono::Utc::now().to_rfc3339();
    let mode_str = format!("{:?}", opts.mode).to_lowercase();
    let mut records: Vec<BenchRecord> = Vec::new();

    for workload in &selected {
        let input_bytes = std::fs::read(&workload.source_path)?;
        let input = String::from_utf8_lossy(&input_bytes);

        for engine_id in &opts.engines {
            let adapter = adapters::get_adapter(engine_id).map_err(|e| e.to_string())?;
            let mut samples: Vec<u128> = Vec::with_capacity(opts.iterations as usize);
            let mut last_diag = 0usize;
            let mut last_hl = 0usize;
            let mut error_class: Option<String> = None;

            for _ in 0..opts.iterations {
                match adapter.run_mode(opts.mode, &input) {
                    Ok(run) => {
                        samples.push(run.elapsed_ns);
                        last_diag = run.diagnostics_count;
                        last_hl = run.highlights_count;
                    }
                    Err(err) => {
                        error_class = Some(classify_error(&err));
                        break;
                    }
                }
            }

            let (exit_status, mean_ns, median_ns, p95_ns, stdev_ns, throughput) =
                if error_class.is_some() {
                    (String::from("error"), 0u128, 0u128, 0u128, 0u128, 0.0f64)
                } else {
                    let (mean, median, p95, stdev) = calc_summary(&samples);
                    let tput = if mean == 0 {
                        0.0
                    } else {
                        (workload.bytes as f64) / (mean as f64 / 1_000_000_000.0)
                    };
                    (String::from("success"), mean, median, p95, stdev, tput)
                };

            records.push(BenchRecord {
                run_id: run_id.clone(),
                timestamp_utc: timestamp_utc.clone(),
                git_sha: ctx.git_sha.clone(),
                engine: adapter.id().to_string(),
                profile: workload.profile.clone(),
                mode: format!("compare-{mode_str}"),
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
    }

    print_speedup_table(&records, &opts.engines, &selected);
    report::persist_bench_records(ctx, "compare", &records)?;
    println!("compare completed: {} record(s)", records.len());
    Ok(())
}

fn print_speedup_table(records: &[BenchRecord], engines: &[String], workloads: &[&Workload]) {
    let baseline = &engines[0];
    println!();
    println!(
        "Speedup table  (baseline = {baseline}, ratio = baseline_mean / engine_mean, >1.0x means engine is faster)"
    );
    println!(
        "{:<45} {:<22} {:>12} {:>12} {:>10}",
        "workload", "engine", "mean_ns", "stdev_ns", "ratio"
    );
    println!("{}", "-".repeat(106));

    for workload in workloads {
        let base_mean = records
            .iter()
            .find(|r| r.workload_id == workload.id && &r.engine == baseline)
            .map(|r| r.mean_ns)
            .unwrap_or(0);

        for engine_id in engines {
            if let Some(rec) = records
                .iter()
                .find(|r| r.workload_id == workload.id && &r.engine == engine_id)
            {
                let ratio = if rec.mean_ns == 0 || base_mean == 0 {
                    String::from("  n/a")
                } else {
                    format!("{:.2}x", base_mean as f64 / rec.mean_ns as f64)
                };
                let mean_display = if rec.exit_status == "error" {
                    String::from("ERROR")
                } else {
                    rec.mean_ns.to_string()
                };
                let stdev_display = if rec.exit_status == "error" {
                    String::from("-")
                } else {
                    rec.stdev_ns.to_string()
                };
                println!(
                    "{:<45} {:<22} {:>12} {:>12} {:>10}",
                    workload.id, engine_id, mean_display, stdev_display, ratio
                );
            }
        }
        println!();
    }
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
