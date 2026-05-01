pub mod benchmark;
pub mod compare;
pub mod regression;
pub mod report_cmd;
pub mod stress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchRecord {
    pub run_id: String,
    pub timestamp_utc: String,
    pub git_sha: String,
    pub engine: String,
    pub profile: String,
    pub mode: String,
    pub workload_id: String,
    pub workload_bytes: usize,
    pub workload_sha256: String,
    pub iterations: u32,
    pub mean_ns: u128,
    pub median_ns: u128,
    pub p95_ns: u128,
    pub stdev_ns: u128,
    pub throughput_bytes_s: f64,
    pub exit_status: String,
    pub error_class: Option<String>,
    pub diagnostics_count: usize,
    pub highlights_count: usize,
}

pub fn calc_summary(samples: &[u128]) -> (u128, u128, u128, u128) {
    if samples.is_empty() {
        return (0, 0, 0, 0);
    }

    let mut sorted = samples.to_vec();
    sorted.sort_unstable();

    let len = sorted.len();
    let sum: u128 = sorted.iter().copied().sum();
    let mean = sum / len as u128;
    let median = if len % 2 == 0 {
        let hi = sorted[len / 2];
        let lo = sorted[(len / 2) - 1];
        (lo + hi) / 2
    } else {
        sorted[len / 2]
    };
    let p95_rank = ((len * 95) + 99) / 100;
    let p95_index = p95_rank.saturating_sub(1).min(len - 1);
    let p95 = sorted[p95_index];

    let mut variance_acc: u128 = 0;
    for sample in &sorted {
        let delta = sample.abs_diff(mean);
        variance_acc = variance_acc.saturating_add(delta.saturating_mul(delta));
    }
    let variance = variance_acc / len as u128;
    let stdev = int_sqrt(variance);

    (mean, median, p95, stdev)
}

fn int_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
