#!/usr/bin/env bash
# run-stress.sh — run the perf-lab stress harness.
#
# Usage: ./scripts/run-stress.sh [--release] [perf-lab stress options...]
#
# Examples:
#   ./scripts/run-stress.sh --mode e2e --loops 100
#   ./scripts/run-stress.sh --mode parse --workload fixture:pathological:generated-synthetic.md
#   ./scripts/run-stress.sh --continue-on-error --list

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

"tools/perf-lab/target/${PROFILE}/perf-lab" stress "$@"
