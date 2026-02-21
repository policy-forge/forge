#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

run_step() {
    local label="$1"
    shift
    echo "[ci-local] ${label}"
    "$@"
}

cd "${REPO_ROOT}"

run_step "cargo fmt --check" cargo fmt --check
run_step "cargo clippy -- -D warnings" cargo clippy -- -D warnings
run_step "cargo test" cargo test
run_step "cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3" \
    cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3
run_step "cargo audit" cargo audit
run_step "cargo deny check" cargo deny check

echo "[ci-local] all checks passed"
