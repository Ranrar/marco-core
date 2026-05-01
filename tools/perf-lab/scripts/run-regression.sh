#!/usr/bin/env bash
# run-regression.sh — compare a current perf-lab JSON artifact against a baseline.
#
# Usage:
#   ./scripts/run-regression.sh --baseline PATH --current PATH [options]
#
# Options:
#   --baseline PATH     Baseline perf-lab JSON artifact (required)
#   --current PATH      Current run perf-lab JSON artifact (required)
#   --warn PCT          Warn threshold % (default: 10)
#   --fail PCT          Fail threshold % (default: 20)
#   --min-failures N    Min regressions to hard-fail (default: 2)
#   --critical-only     Gate only on critical workloads
#   --release           Build in release mode
#   -h, --help          Show this help
#
# Quick start (run a fresh bench first, then compare to stored baseline):
#   ./scripts/run-bench.sh --release --mode e2e --iterations 30
#   LATEST=$(ls -1t tools/perf-lab/output/summary/bench-*.json | head -1)
#   ./scripts/run-regression.sh \
#     --baseline tools/perf-lab/output/baseline/bench-baseline.json \
#     --current "$LATEST"
#
# Capture a new baseline:
#   cp "$LATEST" tools/perf-lab/output/baseline/bench-baseline.json

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../" && pwd)"
cd "$ROOT"

BASELINE=""
CURRENT=""
WARN="10"
FAIL_T="20"
MIN_FAILURES="2"
CRITICAL_ONLY=""
RELEASE_FLAG=""
PROFILE="debug"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --baseline)     BASELINE="$2";    shift 2 ;;
    --current)      CURRENT="$2";     shift 2 ;;
    --warn)         WARN="$2";        shift 2 ;;
    --fail)         FAIL_T="$2";      shift 2 ;;
    --min-failures) MIN_FAILURES="$2"; shift 2 ;;
    --critical-only) CRITICAL_ONLY="--critical-only"; shift ;;
    --release)      RELEASE_FLAG="--release"; PROFILE="release"; shift ;;
    -h|--help)
      grep '^#' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$BASELINE" || -z "$CURRENT" ]]; then
  echo "ERROR: --baseline and --current are both required." >&2
  echo "Run with -h for usage." >&2
  exit 1
fi

cargo build --manifest-path tools/perf-lab/Cargo.toml $RELEASE_FLAG 2>&1 | tail -2

"tools/perf-lab/target/${PROFILE}/perf-lab" regression \
  --baseline "$BASELINE" \
  --current  "$CURRENT" \
  --warn-threshold "$WARN" \
  --fail-threshold "$FAIL_T" \
  --min-failures   "$MIN_FAILURES" \
  $CRITICAL_ONLY
