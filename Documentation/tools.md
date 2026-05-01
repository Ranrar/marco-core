# perf-lab — Performance and Stress Tooling

`tools/perf-lab/` is a standalone `publish = false` Rust crate that benchmarks
and stress-tests `marco-core` and compares it against third-party Markdown
engines. It is **not** part of the published library; it is developer tooling
only.

> This is internal developer documentation. It is **not** shipped with the
> crate (`.dev/` is excluded from the published `.crate` tarball).

---

## Quick start

```bash
# Build (debug)
cargo build --manifest-path tools/perf-lab/Cargo.toml

# Run a quick end-to-end benchmark
./tools/perf-lab/scripts/run-bench.sh --mode e2e --iterations 30

# Compare marco-core vs pulldown-cmark vs comrak
./tools/perf-lab/scripts/run-compare.sh --mode e2e --iterations 20

# Stress harness (100 loops per workload)
./tools/perf-lab/scripts/run-stress.sh --mode e2e --loops 100

# Cross-engine wall-clock comparison with hyperfine
./tools/perf-lab/scripts/run-hyperfine.sh --release

# Regression gate (compare two artifacts)
./tools/perf-lab/scripts/run-regression.sh \
  --baseline tools/perf-lab/output/baseline/bench-baseline.json \
  --current  tools/perf-lab/output/summary/bench-<timestamp>.json
```

---

## Architecture

```
tools/perf-lab/
├── Cargo.toml          — standalone crate, path dep on marco-core
├── perf-lab.ron        — runtime config (RON format)
├── benches/
│   └── marco_core_modes.rs   — Criterion harness (spawned as subprocess)
├── fixtures/           — local Markdown fixtures + generated synthetics
├── output/             — all artifacts land here (git-ignored except .gitkeep)
│   ├── baseline/       — committed baseline manifest + optional baseline JSON
│   ├── hyperfine/      — hyperfine JSON exports
│   └── summary/        — bench/stress/compare output (JSON/CSV/Markdown)
├── scripts/            — thin shell wrappers (see §Scripts below)
└── src/
    ├── main.rs         — CLI entry point, dispatches to runners
    ├── cli.rs          — all clap structs (Args, Command, …Options, Mode, …)
    ├── config.rs       — PerfLabConfig (loaded from perf-lab.ron)
    ├── workloads.rs    — workload discovery, manifest I/O, drift detection
    ├── adapters/       — one file per engine adapter
    └── runners/        — one file per subcommand
        report/         — CSV / JSON / Markdown report emitters
```

---

## Configuration — `perf-lab.ron`

```ron
(
  spec_dir: "tools/spec",
  local_fixtures_dir: "tools/perf-lab/fixtures",
  output_dir: "tools/perf-lab/output",
  baseline_manifest: Some("tools/perf-lab/output/baseline/workload-manifest-v1.json"),
  profiles: ["commonmark-core", "gfm-core", "marco-extensions"],
  default_engine: "marco-core",
  strict: false,
  synthetic_enabled: true,
  synthetic_seed: 1337,
)
```

All paths are relative to the repo root. `synthetic_enabled: true` materialises
a set of pseudo-random Markdown fixtures (seeded for reproducibility) on each
run.

---

## Subcommands

### `bench`

Runs the primary benchmark loop. For each engine × workload × mode
combination it executes `iterations` samples, collects per-sample nanosecond
timings, and computes `mean / median / p95 / stdev` via integer arithmetic.
Results are written to `output/summary/bench-<ISO8601>.{json,csv,md}`.

```
perf-lab bench [OPTIONS]

  --engine ENGINE         Engine to benchmark (default: from config)
  --workload ID           Run a single workload by ID (default: all)
  --mode MODE             parse | render | e2e | intelligence (default: e2e)
  --iterations N          Samples per workload (default: 1)
  --criterion             Use Criterion backend instead of direct timing
  --criterion-sample-size N   Min 10 (default: 10)
  --criterion-warmup-ms MS    (default: 200)
  --criterion-measurement-ms MS  (default: 500)
  --manifest-out PATH     Write current workload manifest to PATH
  --baseline-manifest PATH    Override config baseline_manifest
  --check-manifest-drift  Fail if workloads changed vs baseline manifest
  --list                  Print available workloads and exit
```

**`BenchRecord` fields written to JSON:**

| Field | Type | Description |
|---|---|---|
| `run_id` | UUID | Unique ID for this bench run |
| `timestamp_utc` | ISO-8601 | Wall-clock time of run |
| `git_sha` | string | `git rev-parse HEAD` (empty if unavailable) |
| `engine` | string | Engine adapter ID |
| `profile` | string | Workload profile tag |
| `mode` | string | `parse` / `render` / `e2e` / `intelligence` |
| `workload_id` | string | `fixture:<profile>:<filename>` |
| `workload_bytes` | usize | Source file size |
| `workload_sha256` | string | Hex SHA-256 of source file |
| `iterations` | u32 | Number of samples collected |
| `mean_ns` | u128 | Arithmetic mean of samples (ns) |
| `median_ns` | u128 | Median of samples (ns) |
| `p95_ns` | u128 | 95th-percentile of samples (ns) |
| `stdev_ns` | u128 | Integer square-root of variance (ns) |
| `throughput_bytes_s` | f64 | `workload_bytes / mean_ns * 1e9` |
| `exit_status` | string | `ok` / `error` / `partial-error` |
| `error_class` | Option\<string\> | `unsupported-mode` / `panic` / `timeout` / `oom` / `engine-error` |
| `diagnostics_count` | usize | Number of diagnostics returned (intelligence mode) |
| `highlights_count` | usize | Number of highlight spans returned (intelligence mode) |

---

### `stress`

Runs many loops of a single engine/workload/mode, accumulates per-sample
timing, and prints a progress line per workload. Use this to hunt for memory
leaks, panics under load, and throughput degradation over time.

```
perf-lab stress [OPTIONS]

  --engine ENGINE
  --workload ID
  --mode MODE            (default: e2e)
  --loops N              Iterations per workload (default: 10)
  --continue-on-error    Record errors and continue instead of aborting
  --list
```

Progress line format:

```
stress <workload_id> | engine=marco-core mode=e2e loops=100 mean=157ns p95=180ns tput=6.3MB/s status=ok
```

Exits non-zero if any error occurs and `--continue-on-error` is not set.

---

### `compare`

Runs multiple engines in-process against the same workloads and prints a
speedup table. The first `--engine` is used as the baseline (ratio = 1.00×).
Ratios > 1.00× mean the engine is faster than the baseline.

```
perf-lab compare [OPTIONS]

  --engine ENGINE        Repeatable; can specify multiple engines
  --workload ID
  --mode MODE            (default: e2e)
  --iterations N         Samples per engine per workload (default: 10)
  --list
```

Default engines (when none specified): `marco-core`, `pulldown-cmark`, `comrak`.

Speedup table columns: `workload | engine | mean_ns | stdev_ns | ratio`.

---

### `report`

Re-renders an existing bench artifact in a different format, or ingests a
hyperfine JSON export and converts it to `BenchRecord` format.

```
perf-lab report [OPTIONS]

  --input PATH           perf-lab JSON artifact to re-render
  --hyperfine-input PATH hyperfine --export-json file to ingest
  --output PATH          Write to file instead of stdout
  --format FORMAT        json | csv | markdown (default: markdown)
```

**Hyperfine ingestion:** Extracts engine/workload/mode from the hyperfine
`command` string by parsing `--engine`, `--workload`, and `--mode` flags.
Converts `mean`/`stddev`/`median` from seconds → nanoseconds. Computes p95
from the `times` array (ceiling index).

---

### `regression`

Compares two `BenchRecord` JSON artifacts (baseline vs current). Matches
records by `engine + workload_id + mode`. Computes `change_pct` and assigns
a severity:

- **Ok** — change ≤ `--warn-threshold`
- **Warn** — change > `--warn-threshold` and ≤ `--fail-threshold`; exits 0
- **Fail** — change > `--fail-threshold`; exits 1 when `fail_count >= --min-failures`

```
perf-lab regression [OPTIONS]

  --baseline PATH        Baseline BenchRecord JSON artifact (required)
  --current PATH         Current run BenchRecord JSON artifact (required)
  --warn-threshold PCT   % regression to warn (default: 10)
  --fail-threshold PCT   % regression to fail (default: 20)
  --min-failures N       Minimum failing records to exit 1 (default: 2)
  --critical-only        Only gate on records where profile/id contains "critical"
```

Output:

```
Regression check
  baseline : path/to/baseline.json
  current  : path/to/current.json
  thresholds: warn >10%  fail >20%  min-failures 2

workload              engine       mode    baseline_ns  current_ns  change%  status
──────────────────────────────────────────────────────────────────────────────────
fixture:small:…       marco-core   e2e          157233      157038     -0.1%  ok

Summary: 1 ok / 0 warn / 0 fail  (checked 1 record(s))
regression gate passed
```

---

## Engine Adapters (`src/adapters/`)

| ID | File | Modes | Notes |
|----|------|-------|-------|
| `marco-core` | `marco_core.rs` | parse / render / e2e / intelligence | Native in-process; uses `sanitize_input`, `parse`, `render`, `MarkdownIntelligenceProvider` |
| `pulldown-cmark` | `pulldown_cmark.rs` | parse / render / e2e | Uses `Options::all()` for GFM; `Parser::new_ext().count()` for parse, `html::push_html` for render |
| `comrak` | `comrak.rs` | parse / render / e2e | Uses arena allocator; `parse_document` for parse, `format_html` (String output) for render |
| `markdown-rs` | `markdown_rs.rs` | — | Placeholder; returns `unsupported-mode` |
| `markdown-it-rs` | `markdown_it.rs` | — | Placeholder; returns `unsupported-mode` |

All adapters implement the `EngineAdapter` trait:

```rust
pub trait EngineAdapter {
    fn id(&self) -> &'static str;
    fn run_mode(&self, mode: Mode, input: &str) -> Result<EngineRun, String>;
}
```

`EngineRun` carries `elapsed_ns`, `diagnostics_count`, and `highlights_count`.

---

## Workloads (`src/workloads.rs`)

Workloads are discovered at startup from two sources:

1. **Spec fixtures** — `tools/spec/*.md` files, tagged with the profile derived
   from the file name.
2. **Local fixtures** — `tools/perf-lab/fixtures/**/*.md`; includes a
   `generated-synthetic.md` created deterministically from `synthetic_seed`.

Each `Workload` carries:

```rust
pub struct Workload {
    pub id: String,       // "fixture:<profile>:<filename>"
    pub profile: String,  // "commonmark-core" | "gfm-core" | "marco-extensions"
    pub source_path: PathBuf,
    pub bytes: usize,
    pub sha256: String,   // hex SHA-256 for drift detection
}
```

**Manifest drift detection** compares the current workload list against a
baseline manifest JSON (`output/baseline/workload-manifest-v1.json`) and
reports missing, unexpected, and changed (SHA-256) IDs. When
`--check-manifest-drift` is set, any drift causes a non-zero exit.

---

## Scripts (`tools/perf-lab/scripts/`)

All scripts accept `--release` as the **first** argument to build and run in
release mode (recommended for fair cross-engine comparison). They all resolve
the repo root as `$(dirname "${BASH_SOURCE[0]}")/../../../`.

| Script | Purpose | Key flags |
|--------|---------|-----------|
| `run-bench.sh` | Run `perf-lab bench` | `[--release] [bench options…]` |
| `run-stress.sh` | Run `perf-lab stress` | `[--release] [stress options…]` |
| `run-compare.sh` | Run `perf-lab compare` (defaults to all 3 engines) | `[--release] [compare options…]` |
| `run-hyperfine.sh` | Process-level comparison via hyperfine | `--workload`, `--mode`, `--warmup`, `--runs`, `--engines`, `--out`, `--release` |
| `run-regression.sh` | Run `perf-lab regression` | `--baseline PATH`, `--current PATH`, `--warn`, `--fail`, `--min-failures`, `--critical-only`, `--release` |

### `run-hyperfine.sh` details

Requires `hyperfine >= 1.18` in `PATH`. Builds the perf-lab binary, spawns
`hyperfine --shell none --export-json` with one command per engine, writes a
dated JSON file to `output/hyperfine/` and copies it to `latest.json`, then
automatically invokes `perf-lab report --hyperfine-input` to produce a
Markdown summary.

```bash
# All three engines, release build, 20 runs
./scripts/run-hyperfine.sh --release --runs 20 --warmup 5

# Install hyperfine if missing
cargo install hyperfine
```

### `run-regression.sh` workflow

```bash
# 1. Capture a baseline (first time or after a release)
./scripts/run-bench.sh --release --mode e2e --iterations 30
cp $(ls -1t tools/perf-lab/output/summary/bench-*.json | head -1) \
   tools/perf-lab/output/baseline/bench-baseline.json

# 2. Later, after code changes, compare
./scripts/run-bench.sh --release --mode e2e --iterations 30
./scripts/run-regression.sh \
  --release \
  --baseline tools/perf-lab/output/baseline/bench-baseline.json \
  --current  $(ls -1t tools/perf-lab/output/summary/bench-*.json | head -1)
```

---

## CI — `.github/workflows/ci-perf.yml`

Two-stage workflow triggered on pushes to `main` and pull requests.

### Stage A — non-blocking artifact collection (push to `main` only)

1. Build perf-lab in release mode.
2. Run `bench --engine marco-core --mode e2e --iterations 30` and
   `bench --engine marco-core --mode parse --iterations 30`.
3. Upload the most recent summary JSON as a GitHub Actions artifact
   (`perf-bench-main-<sha>`, retention 30 days).

### Stage B — regression gate (pull request only)

1. Build perf-lab in release mode.
2. Run `bench --engine marco-core --mode e2e --iterations 20` on the PR branch.
3. Attempt to download the Stage A artifact for `base.sha`.
4. If the baseline artifact is found, run `perf-lab regression` with
   `--warn-threshold 10 --fail-threshold 20 --min-failures 2`.
   - Exit 1 if ≥ 2 workloads regress by > 20 %.
   - Exit 0 (with WARN lines) if regressions are 10–20 %.
5. If no baseline artifact is found (first run, expired TTL) the gate is
   skipped gracefully with a notice.
6. The PR branch artifact is always uploaded (`retention-days: 14`).

**System dependencies** installed on both stages:

```bash
sudo apt-get install -y build-essential pkg-config libfontconfig-dev
```

---

## Statistics helper — `calc_summary`

```rust
pub fn calc_summary(samples: &[u128]) -> (mean, median, p95, stdev)
```

- All arithmetic uses integer `u128` to avoid float precision drift.
- p95: ceiling-rank index (`(n * 95 + 99) / 100 - 1`, clamped to `n-1`).
- Stdev: integer square root of `Σ(delta²) / n` via Newton's method.

---

## Output directory layout

```
tools/perf-lab/output/
├── baseline/
│   ├── workload-manifest-v1.json   — committed; used for drift detection
│   └── bench-baseline.json         — optional committed baseline for regression gate
├── hyperfine/
│   ├── .gitkeep
│   ├── latest.json                 — symlink/copy of most recent hyperfine run
│   └── hyperfine-<timestamp>.json  — dated export
└── summary/
    ├── bench-<timestamp>.json
    ├── bench-<timestamp>.csv
    └── bench-<timestamp>.md
```

Everything under `output/` except `baseline/` and `.gitkeep` files is
git-ignored.
