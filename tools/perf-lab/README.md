# perf-lab

Standalone `publish = false` performance and regression benchmarking crate for
`marco-core`. Not part of the published library — developer tooling only.

See [`Documentation/tools.md`](../../Documentation/tools.md) for the full
reference (subcommands, options, BenchRecord schema, CI workflow, scripts).

## Quick start

```bash
# End-to-end benchmark (release build recommended)
./tools/perf-lab/scripts/run-bench.sh --release --mode e2e --iterations 30

# Cross-engine comparison: marco-core vs pulldown-cmark vs comrak
./tools/perf-lab/scripts/run-compare.sh --release --mode e2e --iterations 20

# Stress harness
./tools/perf-lab/scripts/run-stress.sh --release --mode e2e --loops 100

# Regression gate (compare two bench artifacts)
./tools/perf-lab/scripts/run-regression.sh --release \
  --baseline tools/perf-lab/output/baseline/bench-baseline.json \
  --current  tools/perf-lab/output/summary/bench-<timestamp>.json
```

## Subcommands

| Subcommand | Purpose |
|---|---|
| `bench` | Direct-timing benchmark; writes JSON/CSV/Markdown artifacts |
| `stress` | Repeated-loop harness; checks for panics and throughput drift |
| `compare` | In-process speedup table against pulldown-cmark and comrak |
| `report` | Re-renders an artifact or ingests a hyperfine JSON export |
| `regression` | Loads two `BenchRecord` JSON files; exits 1 on regression |

## Engine adapters

| ID | Modes |
|---|---|
| `marco-core` | parse / render / e2e / intelligence |
| `marco-core-raw` | parse / render / e2e (positions, math, diagrams disabled) |
| `pulldown-cmark` | parse / render / e2e |
| `comrak` | parse / render / e2e |
| `markdown-rs` | placeholder |
| `markdown-it-rs` | placeholder |

## Build

```bash
cargo build --manifest-path tools/perf-lab/Cargo.toml --release
cargo test  --manifest-path tools/perf-lab/Cargo.toml
```
