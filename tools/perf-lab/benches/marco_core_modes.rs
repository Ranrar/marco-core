use criterion::{criterion_group, criterion_main, Criterion};
use std::env;
use std::fs;
use std::hint::black_box;

#[derive(Clone, Copy, Debug)]
enum BenchMode {
    Parse,
    ParseMinimal,
    Render,
    E2e,
    E2eMinimal,
    Intelligence,
}

impl BenchMode {
    fn from_env() -> Result<Self, String> {
        let raw = env::var("PERF_LAB_MODE").map_err(|_| "PERF_LAB_MODE is required".to_string())?;
        match raw.as_str() {
            "parse" => Ok(Self::Parse),
            "parse-minimal" => Ok(Self::ParseMinimal),
            "render" => Ok(Self::Render),
            "e2e" => Ok(Self::E2e),
            "e2e-minimal" => Ok(Self::E2eMinimal),
            "intelligence" => Ok(Self::Intelligence),
            _ => Err(format!("unsupported PERF_LAB_MODE: {raw}")),
        }
    }
}

fn bench(c: &mut Criterion) {
    let engine = env::var("PERF_LAB_ENGINE").unwrap_or_else(|_| String::from("marco-core"));
    if engine != "marco-core" {
        panic!("criterion bench currently supports only marco-core engine");
    }

    let mode = BenchMode::from_env().expect("PERF_LAB_MODE must be set to parse/render/e2e/intelligence");
    let workload_path = env::var("PERF_LAB_WORKLOAD_PATH").expect("PERF_LAB_WORKLOAD_PATH is required");
    let bench_id = env::var("PERF_LAB_BENCH_ID").expect("PERF_LAB_BENCH_ID is required");

    let input = fs::read_to_string(workload_path).expect("failed to read workload file");

    c.bench_function(&bench_id, |b| {
        b.iter(|| {
            run_mode(mode, black_box(&input)).expect("mode run failed");
        })
    });
}

fn run_mode(mode: BenchMode, input: &str) -> Result<(), String> {
    match mode {
        BenchMode::Parse => {
            let _doc = marco_core::parse(input).map_err(|e| e.to_string())?;
            Ok(())
        }
        BenchMode::ParseMinimal => {
            // No position tracking, no math, no diagrams — minimal hot-path work.
            let opts = marco_core::ParseOptions {
                track_positions: false,
                parse_math: false,
                parse_diagrams: false,
            };
            let _doc = marco_core::parse_with_options(input, opts).map_err(|e| e.to_string())?;
            Ok(())
        }
        BenchMode::Render => {
            let doc = marco_core::parse(input).map_err(|e| e.to_string())?;
            let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        BenchMode::E2e => {
            let doc = marco_core::parse(input).map_err(|e| e.to_string())?;
            let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        BenchMode::E2eMinimal => {
            // Parse with minimal options, then render as normal.
            let opts = marco_core::ParseOptions {
                track_positions: false,
                parse_math: false,
                parse_diagrams: false,
            };
            let doc = marco_core::parse_with_options(input, opts).map_err(|e| e.to_string())?;
            let _html = marco_core::render(&doc, &marco_core::RenderOptions::default())
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        BenchMode::Intelligence => {
            let doc = marco_core::parse(input).map_err(|e| e.to_string())?;
            let mut provider = marco_core::MarkdownIntelligenceProvider::new();
            provider.update_document(doc);
            let _ = provider.diagnostics();
            let _ = provider.highlights(input);
            Ok(())
        }
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
