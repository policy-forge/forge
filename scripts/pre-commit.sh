#!/usr/bin/env bash

set -euo pipefail

SOURCE="${BASH_SOURCE[0]}"
while [[ -h "${SOURCE}" ]]; do
    SOURCE_DIR="$(cd -P "$(dirname "${SOURCE}")" && pwd)"
    SOURCE="$(readlink "${SOURCE}")"
    [[ "${SOURCE}" != /* ]] && SOURCE="${SOURCE_DIR}/${SOURCE}"
done
SCRIPT_DIR="$(cd -P "$(dirname "${SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [[ "${SKIP_FORGE_PRECOMMIT:-0}" == "1" ]]; then
    if [[ -n "${CI:-}" ]]; then
        echo "[pre-commit] SKIP_FORGE_PRECOMMIT=1 is forbidden in CI" >&2
        exit 1
    fi
    echo "[pre-commit] SKIP_FORGE_PRECOMMIT=1, skipping local checks" >&2
    exit 0
fi

run_step() {
    local label="$1"
    shift
    echo "[pre-commit] ${label}"
    "$@"
}

cd "${REPO_ROOT}"

# Gates run against the working tree; with unstaged changes present they would
# validate a tree that differs from the committed snapshot (F0903).
if ! git diff --quiet; then
    echo "[pre-commit] unstaged changes present — stage or stash them first so the gates validate the commit contents" >&2
    exit 1
fi
if git diff --cached --quiet; then
    echo "[pre-commit] nothing staged — skipping checks"
    exit 0
fi

run_step "cargo fmt --check" cargo fmt --check
run_step "cargo clippy --locked -- -D warnings" cargo clippy --locked -- -D warnings
run_step "cargo test --locked" cargo test --locked

require_cargo_subcommands() {
    local subcommand
    for subcommand in "$@"; do
        if ! cargo "${subcommand}" --version >/dev/null 2>&1; then
            echo "[pre-commit] 'cargo ${subcommand}' is not installed; install cargo-${subcommand} --locked" >&2
            exit 1
        fi
    done
}

if [[ "${FORGE_PRECOMMIT_STRICT:-0}" == "1" ]]; then
    require_cargo_subcommands audit deny
    run_step "cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3" \
        cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3
    run_step "cargo audit" cargo audit
    run_step "cargo deny check" cargo deny check
fi

echo "[pre-commit] all checks passed"
