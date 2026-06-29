#!/usr/bin/env bash
# Measure test coverage for zhihu-cli.
#
# Usage:
#   ./scripts/coverage.sh           # summary + gate check
#   ./scripts/coverage.sh html      # HTML report in target/llvm-cov/html
#   ./scripts/coverage.sh text      # per-line annotated source
#   ./scripts/coverage.sh lcov      # lcov.info for external tooling
#
# Gate: total line coverage must be >= 80% (configurable via COVERAGE_MIN).
# Override with: COVERAGE_MIN=85 ./scripts/coverage.sh

set -euo pipefail

# Repo root (one level up from this script).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Configurable threshold.
COVERAGE_MIN="${COVERAGE_MIN:-80}"

# Mode defaults to summary + gate.
MODE="${1:-summary}"

# cargo-llvm-cov: install with `cargo install cargo-llvm-cov`.
# llvm-tools-preview component is required for the underlying coverage runtime.
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
    echo "error: cargo-llvm-cov not found." >&2
    echo "       install with: cargo install cargo-llvm-cov" >&2
    echo "       and:          rustup component add llvm-tools-preview" >&2
    exit 1
fi

case "$MODE" in
    summary)
        echo "Coverage summary (gate: total line coverage >= ${COVERAGE_MIN}%):"
        cargo llvm-cov --all-targets --no-cfg-coverage --summary-only
        cargo llvm-cov --all-targets --no-cfg-coverage \
            --fail-under-lines "$COVERAGE_MIN" >/dev/null
        echo ""
        echo "PASS: line coverage meets the ${COVERAGE_MIN}% gate."
        ;;
    html)
        echo "Generating HTML report at target/llvm-cov/html/index.html ..."
        cargo llvm-cov --all-targets --no-cfg-coverage --html --output-dir target/llvm-cov
        echo "Open: target/llvm-cov/html/index.html"
        ;;
    text)
        cargo llvm-cov --all-targets --no-cfg-coverage --text --show-missing-lines
        ;;
    lcov)
        cargo llvm-cov --all-targets --no-cfg-coverage --lcov --output-path lcov.info
        echo "Wrote lcov.info"
        ;;
    *)
        echo "usage: $0 [summary|html|text|lcvo]" >&2
        exit 2
        ;;
esac
