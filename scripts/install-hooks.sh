#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
HOOKS_DIR="$(git -C "${REPO_ROOT}" rev-parse --path-format=absolute --git-path hooks)" || {
    echo "[install-hooks] not inside a git repository" >&2
    exit 1
}
PRE_COMMIT_HOOK="${HOOKS_DIR}/pre-commit"

if [[ ! -d "${HOOKS_DIR}" ]]; then
    echo "[install-hooks] hooks directory not found: ${HOOKS_DIR}" >&2
    exit 1
fi

cat > "${PRE_COMMIT_HOOK}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
exec "${REPO_ROOT}/scripts/pre-commit.sh" "$@"
EOF

chmod +x "${PRE_COMMIT_HOOK}"

echo "[install-hooks] installed ${PRE_COMMIT_HOOK}"
echo "[install-hooks] this hook runs scripts/pre-commit.sh on each commit"
