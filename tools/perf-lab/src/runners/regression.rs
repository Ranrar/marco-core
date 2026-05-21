use crate::cli::RegressionOptions;
use crate::runners::BenchRecord;
use crate::AppContext;

#[derive(Debug)]
struct RegressionResult {
    workload_id: String,
    engine: String,
    mode: String,
    baseline_median_ns: u128,
    current_median_ns: u128,
    /// positive = regression, negative = improvement
    change_pct: f64,
    severity: Severity,
}

#[derive(Debug, PartialEq, Eq)]
enum Severity {
    Ok,
    Warn,
    Fail,
}

pub fn run(_ctx: &AppContext, opts: &RegressionOptions) -> Result<(), Box<dyn std::error::Error>> {
    let baseline_content = std::fs::read_to_string(&opts.baseline)
        .map_err(|e| format!("cannot read baseline {}: {e}", opts.baseline.display()))?;
    let current_content = std::fs::read_to_string(&opts.current)
        .map_err(|e| format!("cannot read current {}: {e}", opts.current.display()))?;

    let baseline: Vec<BenchRecord> = serde_json::from_str(&baseline_content)?;
    let current: Vec<BenchRecord> = serde_json::from_str(&current_content)?;

    println!("Regression check");
    println!("  baseline : {}", opts.baseline.display());
    println!("  current  : {}", opts.current.display());
    println!(
        "  thresholds: warn >{:.0}%  fail >{:.0}%  min-failures {}",
        opts.warn_threshold, opts.fail_threshold, opts.min_failures
    );
    if opts.critical_only {
        println!("  scope    : critical workloads only");
    }
    println!();

    let mut results: Vec<RegressionResult> = Vec::new();

    for cur in &current {
        // Skip records not relevant to gating scope
        if opts.critical_only && !is_critical(cur) {
            continue;
        }

        // Find matching baseline record (same engine + workload + mode)
        let base = baseline.iter().find(|b| {
            b.engine == cur.engine && b.workload_id == cur.workload_id && b.mode == cur.mode
        });

        let Some(base) = base else {
            println!(
                "  SKIP {} {} {} — no baseline record",
                cur.engine, cur.mode, cur.workload_id
            );
            continue;
        };

        if base.median_ns == 0 || cur.median_ns == 0 {
            continue;
        }

        let change_pct =
            (cur.median_ns as f64 - base.median_ns as f64) / base.median_ns as f64 * 100.0;

        let severity = if change_pct > opts.fail_threshold {
            Severity::Fail
        } else if change_pct > opts.warn_threshold {
            Severity::Warn
        } else {
            Severity::Ok
        };

        results.push(RegressionResult {
            workload_id: cur.workload_id.clone(),
            engine: cur.engine.clone(),
            mode: cur.mode.clone(),
            baseline_median_ns: base.median_ns,
            current_median_ns: cur.median_ns,
            change_pct,
            severity,
        });
    }

    // ── print table ───────────────────────────────────────────────────────────
    println!(
        "{:<45} {:<18} {:<10} {:>14} {:>14} {:>10} {}",
        "workload", "engine", "mode", "baseline_ns", "current_ns", "change%", "status"
    );
    println!("{}", "-".repeat(118));

    let mut warn_count = 0usize;
    let mut fail_count = 0usize;

    for r in &results {
        let symbol = match r.severity {
            Severity::Ok => "ok",
            Severity::Warn => "WARN",
            Severity::Fail => "FAIL",
        };
        println!(
            "{:<45} {:<18} {:<10} {:>14} {:>14} {:>+9.1}% {}",
            r.workload_id,
            r.engine,
            r.mode,
            r.baseline_median_ns,
            r.current_median_ns,
            r.change_pct,
            symbol
        );
        match r.severity {
            Severity::Warn => warn_count += 1,
            Severity::Fail => fail_count += 1,
            Severity::Ok => {}
        }
    }

    println!();
    println!(
        "Summary: {} ok / {} warn / {} fail  (checked {} record(s))",
        results.iter().filter(|r| r.severity == Severity::Ok).count(),
        warn_count,
        fail_count,
        results.len()
    );

    // ── exit code logic ───────────────────────────────────────────────────────
    if fail_count >= opts.min_failures {
        eprintln!(
            "\nFAIL: {} workload(s) exceeded the {:.0}% regression threshold (min-failures={}).",
            fail_count, opts.fail_threshold, opts.min_failures
        );
        std::process::exit(1);
    }

    if warn_count > 0 {
        eprintln!(
            "\nWARN: {} workload(s) exceeded the {:.0}% warn threshold.",
            warn_count, opts.warn_threshold
        );
        // warn only — exit 0
    }

    println!("regression gate passed");
    Ok(())
}

fn is_critical(rec: &BenchRecord) -> bool {
    rec.profile.contains("critical") || rec.workload_id.contains("critical")
}
