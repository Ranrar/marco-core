#!/usr/bin/env bash
# run-bench.sh — run the marco-core benchmark suite.
#
# Usage: ./scripts/run-bench.sh [--release] [perf-lab bench options...]
#
# Examples:
#   ./scripts/run-bench.sh --mode parse --iterations 50
#   ./scripts/run-bench.sh --release --engine marco-core --mode e2e --criterion
#   ./scripts/run-bench.sh --list

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../" && pwd)"
cd "$ROOT"

RELEASE_FLAG=""
if [[ "${1:-}" == "--release" ]]; then
  RELEASE_FLAG="--release"
  shift
fi

cargo build --manifest-path tools/perf-lab/Cargo.toml $RELEASE_FLAG 2>&1 | tail -2

PROFILE="debug"
[[ -n "$RELEASE_FLAG" ]] && PROFILE="release"

"tools/perf-lab/target/${PROFILE}/perf-lab" bench "$@"
