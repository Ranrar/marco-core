use crate::runners::BenchRecord;
use std::path::Path;

pub fn write_bench_summary(
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
    let mut out = String::from("# perf-lab summary\n\n");
    out.push_str("| engine | mode | workload_id | mean_ns | p95_ns | throughput_bytes_s | status |\n");
    out.push_str("|---|---|---|---:|---:|---:|---|\n");

    for r in records {
        let line = format!(
            "| {} | {} | {} | {} | {} | {:.3} | {} |\n",
            r.engine, r.mode, r.workload_id, r.mean_ns, r.p95_ns, r.throughput_bytes_s, r.exit_status
        );
        out.push_str(&line);
    }

    out
}
