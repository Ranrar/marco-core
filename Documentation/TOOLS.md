# Tools

Developer tooling for `marco-core`. None are published to crates.io.

## Quick start

```bash
# AST inspection
cargo run --manifest-path tools/marco-ast/Cargo.toml -- --text "# Title" --mode ast

# Benchmark (release recommended for fair results)
./tools/perf-lab/scripts/run-bench.sh --release --mode e2e --iterations 30

# Cross-engine comparison
./tools/perf-lab/scripts/run-compare.sh --release --mode e2e --iterations 20

# Stress test
./tools/perf-lab/scripts/run-stress.sh --release --mode e2e --loops 100

# Regression gate
./tools/perf-lab/scripts/run-regression.sh --release \
  --baseline tools/perf-lab/output/baseline/bench-baseline.json \
  --current  tools/perf-lab/output/summary/bench-<timestamp>.json
```

## marco-ast

AST inspection CLI. Parses a Markdown file and prints the parse tree, HTML
output, diagnostics, or highlights. Useful for debugging grammar and parser
changes.

**Modes:** `ast`, `html`, `both`, `intel`

**Input:** File path or `--text "markdown"` or stdin

**Flags:**
- `--spans` — print line/column/byte ranges
- `--excerpts` — print source excerpts next to nodes
- `--utf8` — print UTF-8 and byte-vs-char diagnostics
- `--time` — print timing summary (sanitize/parse/render/intel)
- `--json` — emit machine-readable JSON report
- `--interactive` — REPL mode (`:mode`, `:clear`, `:help`, `:quit`)

**Example:**
```bash
cargo run --manifest-path tools/marco-ast/Cargo.toml -- \
  path/to/file.md --mode both --spans --time
```

## perf-lab

Performance and regression benchmarking harness. Measures parse, render, and
e2e latency. Compares `marco-core` against `pulldown-cmark` and `comrak`.

**Configuration:** `tools/perf-lab/perf-lab.ron` (workload discovery, output paths, profiles).

**Workloads:** Discovered from `tools/spec/*.json` and `tools/perf-lab/fixtures/*.md`.

### Subcommands

#### `bench` — Direct-timing benchmark

Runs iterations of a workload and computes `mean / median / p95 / stdev`.
Writes JSON/CSV/Markdown artifacts to `output/summary/`.

```bash
perf-lab bench [OPTIONS]
  --engine ENGINE         Engine to bench (default: marco-core)
  --workload ID           Single workload (default: all)
  --mode MODE             parse | render | e2e | intelligence (default: e2e)
  --iterations N          Samples per workload (default: 1)
  --criterion             Use Criterion harness instead of direct timing
  --manifest-out PATH     Write workload manifest
  --check-manifest-drift  Fail if workloads changed vs baseline
  --list                  Print available workloads and exit
```

**Output:** `BenchRecord` JSON with `mean_ns`, `median_ns`, `p95_ns`, `stdev_ns`,
`throughput_bytes_s`, `run_id`, `git_sha`, `engine`, `profile`, etc.

#### `stress` — Repeated-loop harness

Loops many times per workload, hunting for panics, leaks, and throughput
degradation.

```bash
perf-lab stress [OPTIONS]
  --engine ENGINE
  --workload ID
  --mode MODE             (default: e2e)
  --loops N               Iterations per workload (default: 10)
  --continue-on-error     Record errors and continue
  --list
```

#### `compare` — In-process speedup table

Runs multiple engines against the same workloads. First engine is baseline (1.00×).

```bash
perf-lab compare [OPTIONS]
  --engine ENGINE         Repeatable; can specify multiple
  --workload ID
  --mode MODE             (default: e2e)
  --iterations N          Samples per engine (default: 10)
  --list
```

Defaults to `marco-core`, `pulldown-cmark`, `comrak`.

#### `report` — Artifact re-render

Converts a bench artifact to a different format or ingests a hyperfine JSON
export.

```bash
perf-lab report [OPTIONS]
  --input PATH           perf-lab JSON artifact
  --hyperfine-input PATH hyperfine JSON export
  --output PATH          Write to file instead of stdout
  --format FORMAT        json | csv | markdown (default: markdown)
```

#### `regression` — Regression gate

Compares baseline vs current artifacts. Exits 1 if median regresses beyond
`--fail-threshold`.

```bash
perf-lab regression [OPTIONS]
  --baseline PATH        Baseline BenchRecord JSON (required)
  --current PATH         Current run BenchRecord JSON (required)
  --warn-threshold PCT   Warn if > this % regression (default: 10)
  --fail-threshold PCT   Fail if > this % regression (default: 20)
  --min-failures N       Min records to fail on (default: 2)
  --critical-only        Gate only on "critical" profiles
```

### Engine adapters

| ID | Modes | Notes |
|---|---|---|
| `marco-core` | parse / render / e2e / intelligence | Native; uses full features |
| `marco-core-raw` | parse / render / e2e | Marco-core with positions/math/diagrams disabled |
| `pulldown-cmark` | parse / render / e2e | Uses `Options::all()` for GFM |
| `comrak` | parse / render / e2e | Uses arena allocator |
| `markdown-rs` | — | Placeholder |
| `markdown-it-rs` | — | Placeholder |

### Scripts

All scripts accept `--release` as the first argument.

```bash
./tools/perf-lab/scripts/run-bench.sh [--release] [bench options]
./tools/perf-lab/scripts/run-stress.sh [--release] [stress options]
./tools/perf-lab/scripts/run-compare.sh [--release] [compare options]
./tools/perf-lab/scripts/run-hyperfine.sh [--release] [options]
./tools/perf-lab/scripts/run-regression.sh [--release] [regression options]
```

**`run-hyperfine.sh`** requires `hyperfine >= 1.18`:
```bash
# 20 runs, 5 warmups, release build
./scripts/run-hyperfine.sh --release --runs 20 --warmup 5
```

Outputs to `tools/perf-lab/output/hyperfine/` and auto-invokes `perf-lab report`
to produce Markdown summary.

### Typical workflow

```bash
# Capture a baseline (after a release or initial setup)
./tools/perf-lab/scripts/run-bench.sh --release --mode e2e --iterations 30
cp $(ls -1t tools/perf-lab/output/summary/bench-*.json | head -1) \
   tools/perf-lab/output/baseline/bench-baseline.json

# Later, after changes, compare
./tools/perf-lab/scripts/run-bench.sh --release --mode e2e --iterations 30
./tools/perf-lab/scripts/run-regression.sh --release \
  --baseline tools/perf-lab/output/baseline/bench-baseline.json \
  --current  $(ls -1t tools/perf-lab/output/summary/bench-*.json | head -1)
```

## Building & testing

```bash
# marco-ast
cargo build --manifest-path tools/marco-ast/Cargo.toml
cargo test  --manifest-path tools/marco-ast/Cargo.toml

# perf-lab
cargo build --manifest-path tools/perf-lab/Cargo.toml --release
cargo test  --manifest-path tools/perf-lab/Cargo.toml

# Extension spec conformance (via perf-lab test runner)
cargo test --manifest-path tools/perf-lab/Cargo.toml --test extension_spec_it
```

## CI — `.github/workflows/ci-perf.yml`

Two-stage performance workflow:

- **Push to `main`:** Run `bench --mode e2e --iterations 30` and `bench --mode parse --iterations 30`.
  Upload `BenchRecord` JSON artifact (30-day retention, keyed by commit SHA).
- **Pull request:** Run `bench --iterations 20` on the PR branch, download base-SHA artifact,
  call `regression --warn-threshold 10 --fail-threshold 20 --min-failures 2`.
  Skip gracefully if no baseline exists.

This gates PRs against a 20% median latency regression on 2+ workloads.
