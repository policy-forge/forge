#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
HOOKS_DIR="$(git -C "${REPO_ROOT}" rev-parse --path-format=absolute --git-path hooks)" || {
    echo "[install-hooks] not inside a git repository" >&2
    exit 1
}
PRE_COMMIT_SCRIPT="${REPO_ROOT}/scripts/pre-commit.sh"

if [[ ! -d "${HOOKS_DIR}" ]]; then
    echo "[install-hooks] hooks directory not found: ${HOOKS_DIR}" >&2
    exit 1
fi

if [[ ! -f "${PRE_COMMIT_SCRIPT}" || ! -x "${PRE_COMMIT_SCRIPT}" ]]; then
    echo "[install-hooks] hook delegate is missing or not executable: ${PRE_COMMIT_SCRIPT}" >&2
    exit 1
fi


if ! cat > "${PRE_COMMIT_HOOK}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
exec "${REPO_ROOT}/scripts/pre-commit.sh" "$@"
EOF
then
    echo "[install-hooks] failed to write ${PRE_COMMIT_HOOK}" >&2
    exit 1
fi

if ! chmod +x "${PRE_COMMIT_HOOK}"; then
    echo "[install-hooks] failed to make ${PRE_COMMIT_HOOK} executable" >&2
    exit 1
fi

echo "[install-hooks] installed ${PRE_COMMIT_HOOK}"
echo "[install-hooks] this hook runs scripts/pre-commit.sh on each commit"
