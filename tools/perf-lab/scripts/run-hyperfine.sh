#!/usr/bin/env bash
# run-hyperfine.sh — cross-engine process-level comparison via hyperfine.
#
# Usage:
#   ./scripts/run-hyperfine.sh [options]
#
# Options:
#   --workload ID    Workload to benchmark (default: fixture:small:generated-synthetic.md)
#   --mode MODE      Bench mode: parse|render|e2e (default: e2e)
#   --warmup N       Hyperfine warmup runs (default: 3)
#   --runs N         Hyperfine measurement runs (default: 15)
#   --engines LIST   Comma-separated engine list (default: marco-core,pulldown-cmark,comrak)
#   --out PATH       JSON output path (default: tools/perf-lab/output/hyperfine/latest.json)
#   --release        Build in release mode (recommended for fair comparison)
#   -h, --help       Show this message
#
# Requirements:
#   hyperfine >= 1.18  (https://github.com/sharkdp/hyperfine)
#
# Examples:
#   # Quick dev compare on small fixture
#   ./scripts/run-hyperfine.sh --workload fixture:small:generated-synthetic.md
#
#   # Release build, all engines, larger fixture
#   ./scripts/run-hyperfine.sh --release --workload spec:commonmark --runs 20 --warmup 5
#
#   # Only marco-core vs pulldown-cmark
#   ./scripts/run-hyperfine.sh --engines marco-core,pulldown-cmark

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT"

# ── defaults ──────────────────────────────────────────────────────────────────
WORKLOAD="fixture:small:generated-synthetic.md"
MODE="e2e"
WARMUP=3
RUNS=15
ENGINES="marco-core,pulldown-cmark,comrak"
OUT_PATH="tools/perf-lab/output/hyperfine/latest.json"
RELEASE_FLAG=""
CARGO_PROFILE="dev"

# ── argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --workload) WORKLOAD="$2"; shift 2 ;;
    --mode)     MODE="$2";     shift 2 ;;
    --warmup)   WARMUP="$2";   shift 2 ;;
    --runs)     RUNS="$2";     shift 2 ;;
    --engines)  ENGINES="$2";  shift 2 ;;
    --out)      OUT_PATH="$2"; shift 2 ;;
    --release)  RELEASE_FLAG="--release"; CARGO_PROFILE="release"; shift ;;
    -h|--help)
      sed -n '/^# /,/^[^#]/p' "$0" | grep '^#' | sed 's/^# \?//'
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

# ── prerequisites check ───────────────────────────────────────────────────────
if ! command -v hyperfine &>/dev/null; then
  echo "ERROR: hyperfine not found in PATH." >&2
  echo "Install with: cargo install hyperfine  OR  brew/apt install hyperfine" >&2
  exit 1
fi

HYPERFINE_VERSION=$(hyperfine --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
echo "hyperfine ${HYPERFINE_VERSION} found"

# ── build perf-lab binary ─────────────────────────────────────────────────────
echo "Building perf-lab ($CARGO_PROFILE)..."
cargo build --manifest-path tools/perf-lab/Cargo.toml $RELEASE_FLAG 2>&1 | tail -3

PERF_LAB_BIN="tools/perf-lab/target/${CARGO_PROFILE}/perf-lab"
if [[ ! -x "$PERF_LAB_BIN" ]]; then
  echo "ERROR: binary not found at $PERF_LAB_BIN" >&2
  exit 1
fi

# ── build hyperfine command list ──────────────────────────────────────────────
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
OUT_DIR="$(dirname "$OUT_PATH")"
mkdir -p "$OUT_DIR"

# Dated copy so we keep history
DATED_OUT="${OUT_DIR}/hyperfine-${TIMESTAMP}.json"

IFS=',' read -ra ENGINE_LIST <<< "$ENGINES"

echo ""
echo "Workload : $WORKLOAD"
echo "Mode     : $MODE"
echo "Engines  : ${ENGINE_LIST[*]}"
echo "Warmup   : $WARMUP  Runs: $RUNS"
echo ""

# Build the list of hyperfine command strings
CMD_ARGS=()
for engine in "${ENGINE_LIST[@]}"; do
  CMD_ARGS+=("${PERF_LAB_BIN} bench --engine ${engine} --mode ${MODE} --iterations 1 --workload '${WORKLOAD}'")
done

# ── run hyperfine ─────────────────────────────────────────────────────────────
hyperfine \
  --warmup "$WARMUP" \
  --runs "$RUNS" \
  --export-json "$DATED_OUT" \
  --shell none \
  "${CMD_ARGS[@]}"

# Symlink/copy to latest
cp "$DATED_OUT" "$OUT_PATH"

echo ""
echo "Hyperfine results:"
echo "  dated   : $DATED_OUT"
echo "  latest  : $OUT_PATH"
echo ""

# ── ingest into perf-lab report pipeline ─────────────────────────────────────
SUMMARY_DIR="tools/perf-lab/output/summary"
mkdir -p "$SUMMARY_DIR"

REPORT_OUT="${SUMMARY_DIR}/hyperfine-${TIMESTAMP}.md"
${PERF_LAB_BIN} report --hyperfine-input "$OUT_PATH" \
  --output "$REPORT_OUT" --format markdown 2>/dev/null \
  && echo "  report  : $REPORT_OUT" \
  || echo "  (report ingestion skipped — run 'perf-lab report --hyperfine-input $OUT_PATH' manually)"
