pub mod csv;
pub mod json;
pub mod markdown;

use crate::runners::BenchRecord;
use crate::AppContext;

pub fn persist_bench_records(
    ctx: &AppContext,
    prefix: &str,
    records: &[BenchRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    let summary_dir = ctx.config.output_dir.join("summary");
    std::fs::create_dir_all(&summary_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let json_path = summary_dir.join(format!("{prefix}-{timestamp}.json"));
    let csv_path = summary_dir.join(format!("{prefix}-{timestamp}.csv"));
    let md_path = summary_dir.join(format!("{prefix}-{timestamp}.md"));

    json::write(&json_path, records)?;
    csv::write_bench_records(&csv_path, records)?;
    markdown::write_bench_summary(&md_path, records)?;

    println!("artifacts:");
    println!("- {}", json_path.display());
    println!("- {}", csv_path.display());
    println!("- {}", md_path.display());

    Ok(())
}
