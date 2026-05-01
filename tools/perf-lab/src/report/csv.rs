use crate::runners::BenchRecord;
use std::path::Path;

pub fn write_bench_records(
    path: &Path,
    records: &[BenchRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, to_string(records))?;
    Ok(())
}

pub fn to_string(records: &[BenchRecord]) -> String {
    let mut out = String::from(
        "run_id,timestamp_utc,git_sha,engine,profile,mode,workload_id,workload_bytes,iterations,mean_ns,median_ns,p95_ns,stdev_ns,throughput_bytes_s,exit_status,error_class,diagnostics_count,highlights_count\n",
    );

    for r in records {
        let error = r
            .error_class
            .as_deref()
            .unwrap_or("")
            .replace('"', "''");
        let line = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{:.3},{},\"{}\",{},{}\n",
            r.run_id,
            r.timestamp_utc,
            r.git_sha,
            r.engine,
            r.profile,
            r.mode,
            r.workload_id,
            r.workload_bytes,
            r.iterations,
            r.mean_ns,
            r.median_ns,
            r.p95_ns,
            r.stdev_ns,
            r.throughput_bytes_s,
            r.exit_status,
            error,
            r.diagnostics_count,
            r.highlights_count,
        );
        out.push_str(&line);
    }

    out
}
