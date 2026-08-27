#!/usr/bin/env bash

set -Eeuo pipefail
trap 'status=$?; echo "[ci-local] FAILED (exit ${status}) at line ${LINENO}: ${BASH_COMMAND}" >&2; exit "${status}"' ERR

SOURCE="${BASH_SOURCE[0]}"
while [[ -h "${SOURCE}" ]]; do
    SOURCE_DIR="$(cd -P "$(dirname "${SOURCE}")" && pwd)"
    SOURCE="$(readlink "${SOURCE}")"
    [[ "${SOURCE}" != /* ]] && SOURCE="${SOURCE_DIR}/${SOURCE}"
done
SCRIPT_DIR="$(cd -P "$(dirname "${SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

run_step() {
    local label="$1"
    shift
    echo "[ci-local] ${label}"
    "$@"
}

cd "${REPO_ROOT}"
require_cargo_subcommands() {
    local subcommand
    for subcommand in "$@"; do
        if ! cargo "${subcommand}" --version >/dev/null 2>&1; then
            echo "[ci-local] 'cargo ${subcommand}' is not installed; install cargo-${subcommand} --locked" >&2
            exit 1
        fi
    done
}

require_cargo_subcommands audit deny vet

run_step "cargo fmt --check" cargo fmt --check
run_step "cargo clippy --all-targets -- -D warnings" cargo clippy --all-targets -- -D warnings
run_step "cargo test" cargo test

if [[ "${CI_LOCAL_BENCH:-0}" == "1" ]]; then
    run_step "cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3" \
        cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3
else
    echo "[ci-local] skipping benchmarks (set CI_LOCAL_BENCH=1 to run)"
fi

run_step "cargo audit" cargo audit
run_step "cargo deny check" cargo deny check
run_step "cargo vet --locked" cargo vet --locked

echo "[ci-local] all checks passed"
