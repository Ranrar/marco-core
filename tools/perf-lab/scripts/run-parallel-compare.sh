#!/usr/bin/env bash
# run-parallel-compare.sh — compare marco-core's opt-in `parallel-render` +
# `parallel-parse` features on vs off.
#
# Cargo features are compile-time and global to a build, so a single
# perf-lab binary cannot toggle parallelism at runtime (see
# .dev/parser-render-optimization-plan.md, Phase 3: "perf-lab's bench
# subcommand doesn't support per-run feature toggling... this was measured
# manually"). This script automates that manual two-binary workaround: build
# perf-lab once with the parallel features off and once with them on, bench
# both against the same workload(s), then feed the two artifacts into the
# existing `regression` subcommand as the diff engine — its per-workload
# %-change table is exactly what "parallel on vs off" needs, no new report
# format required.
#
# Usage:
#   ./scripts/run-parallel-compare.sh [--release] [bench options...]
#   ./scripts/run-parallel-compare.sh [--release] [bench options...] -- [regression options...]
#
# Examples:
#   # Full sweep, all workloads, e2e mode
#   ./scripts/run-parallel-compare.sh --release --mode e2e --iterations 20
#
#   # One workload, parse mode
#   ./scripts/run-parallel-compare.sh --release --mode parse --iterations 20 \
#     --workload fixture:large:paragraph-heavy.md
#
#   # Use as a real gate (fail if parallel ever regresses a workload >30%)
#   ./scripts/run-parallel-compare.sh --release --mode e2e --iterations 20 \
#     -- --warn-threshold 15 --fail-threshold 30 --min-failures 1
#
# By default no gate is applied (thresholds are set high enough that the
# table always prints "regression gate passed") — most workloads are
# expected to be ~unchanged (parallel-parse only defers depth==0 top-level
# content; parallel-render only helps code-block-heavy documents), so a
# tight default threshold would spam warnings on ordinary run-to-run noise.
# Pass your own thresholds after `--` to opt into gating.
#
# The two labeled artifacts (…-sequential.json / …-parallel.json) are kept
# under tools/perf-lab/output/summary/ so the comparison is reproducible
# without re-running.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../" && pwd)"
cd "$ROOT"

RELEASE_FLAG=""
if [[ "${1:-}" == "--release" ]]; then
  RELEASE_FLAG="--release"
  shift
fi
PROFILE="debug"
[[ -n "$RELEASE_FLAG" ]] && PROFILE="release"

# Split args at `--`: everything before goes to `bench` (run identically on
# both binaries), everything after goes to `regression` (gate tuning only).
BENCH_ARGS=()
REGRESSION_ARGS=()
SEEN_SEP=0
for arg in "$@"; do
  if [[ "$arg" == "--" && $SEEN_SEP -eq 0 ]]; then
    SEEN_SEP=1
    continue
  fi
  if [[ $SEEN_SEP -eq 0 ]]; then
    BENCH_ARGS+=("$arg")
  else
    REGRESSION_ARGS+=("$arg")
  fi
done

BIN_DIR="tools/perf-lab/target/${PROFILE}"
SEQ_BIN="${BIN_DIR}/perf-lab-sequential"
PAR_BIN="${BIN_DIR}/perf-lab-parallel"

echo "==> building sequential (parallel-render / parallel-parse off)"
cargo build --manifest-path tools/perf-lab/Cargo.toml $RELEASE_FLAG 2>&1 | tail -2
cp "${BIN_DIR}/perf-lab" "$SEQ_BIN"

echo "==> building parallel (parallel-render + parallel-parse on)"
cargo build --manifest-path tools/perf-lab/Cargo.toml $RELEASE_FLAG \
  --features marco-core/parallel-render,marco-core/parallel-parse 2>&1 | tail -2
cp "${BIN_DIR}/perf-lab" "$PAR_BIN"

extract_json() {
  # perf-lab prints "warning: ...<path>.json" (manifest drift) before the
  # real "artifacts:" block, so anchor specifically on the summary/bench-*
  # path rather than the first *.json seen anywhere in the output.
  grep -oE '[^ ]*/summary/bench-[^ ]*\.json' | tail -1
}

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
SEQ_LABELED="tools/perf-lab/output/summary/parallel-compare-${STAMP}-sequential.json"
PAR_LABELED="tools/perf-lab/output/summary/parallel-compare-${STAMP}-parallel.json"

# Each artifact is copied to its labeled path immediately after its own
# bench run finishes — the two runs can land in the same wall-clock second
# (their filenames are second-precision timestamps), so waiting until both
# finish to copy risks the second run's file silently overwriting the
# first's before it's captured.
echo "==> bench: sequential"
SEQ_OUT="$("$SEQ_BIN" bench "${BENCH_ARGS[@]}" 2>&1)"
echo "$SEQ_OUT" | tail -5
SEQ_JSON="$(echo "$SEQ_OUT" | extract_json)"
[[ -z "$SEQ_JSON" ]] && { echo "ERROR: could not locate a bench JSON artifact in the sequential run's output above (was --list passed?)" >&2; exit 1; }
cp "$SEQ_JSON" "$SEQ_LABELED"

echo "==> bench: parallel"
PAR_OUT="$("$PAR_BIN" bench "${BENCH_ARGS[@]}" 2>&1)"
echo "$PAR_OUT" | tail -5
PAR_JSON="$(echo "$PAR_OUT" | extract_json)"
[[ -z "$PAR_JSON" ]] && { echo "ERROR: could not locate a bench JSON artifact in the parallel run's output above (was --list passed?)" >&2; exit 1; }
cp "$PAR_JSON" "$PAR_LABELED"

echo
echo "==> parallel-render + parallel-parse: on vs off"
echo "    (negative change% = parallel is faster; positive = parallel is slower)"
echo

set +e
"${BIN_DIR}/perf-lab" regression \
  --baseline "$SEQ_LABELED" \
  --current  "$PAR_LABELED" \
  --warn-threshold 1000 \
  --fail-threshold 1000 \
  "${REGRESSION_ARGS[@]}"
REG_EXIT=$?
set -e

echo
echo "artifacts:"
echo "- $SEQ_LABELED"
echo "- $PAR_LABELED"

exit "$REG_EXIT"
