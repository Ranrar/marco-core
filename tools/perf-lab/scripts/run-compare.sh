#!/usr/bin/env bash
# run-compare.sh — cross-engine comparison using the in-process adapter layer.
#
# Usage: ./scripts/run-compare.sh [--release] [perf-lab compare options...]
#
# Examples:
#   ./scripts/run-compare.sh --engine marco-core --engine pulldown-cmark --engine comrak
#   ./scripts/run-compare.sh --engine marco-core --engine comrak --mode parse --iterations 20
#   ./scripts/run-compare.sh --list

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

# Default to all three implemented engines if none specified
if [[ "$*" != *"--engine"* ]]; then
  EXTRA_ENGINES="--engine marco-core --engine pulldown-cmark --engine comrak"
else
  EXTRA_ENGINES=""
fi

exec "tools/perf-lab/target/${PROFILE}/perf-lab" compare $EXTRA_ENGINES "$@"
