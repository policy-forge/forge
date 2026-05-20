#!/usr/bin/env bash
#
# ci/integration-test.sh — End-to-end OSCAL model validation
#
# Generates all available OSCAL models from a test policy using the FORGE CLI,
# validates outputs against OSCAL schemas, diffs against golden fixtures,
# and verifies traceability reports.
#
# Usage: ./ci/integration-test.sh [FORGE_BIN]
#   FORGE_BIN  Path to forge binary (default: auto-detect target/release/forge or cargo run)
#
# Exit codes:
#   0  All checks passed
#   1  A check failed or a required dependency is missing
#
# Dependencies:
#   - forge binary (built from this repo)
#   - xmllint    (optional, for XML schema validation; graceful skip if absent)

set -euo pipefail

# ─── Colour helpers ──────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass()  { echo -e "${GREEN}PASS${NC}  $*"; }
fail()  { echo -e "${RED}FAIL${NC}  $*"; }
skip()  { echo -e "${YELLOW}SKIP${NC}  $*"; }
info()  { echo -e "INFO  $*"; }

# ─── Resolve repo root ──────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# ─── Resolve forge binary ───────────────────────────────────────────────
FORGE="${1:-}"
if [[ -z "${FORGE}" ]]; then
    if [[ -x "target/release/forge" ]]; then
        FORGE="target/release/forge"
    elif [[ -x "target/debug/forge" ]]; then
        FORGE="target/debug/forge"
    else
        # Fall back to cargo run (slower, works when binary not pre-built)
        FORGE="cargo run --quiet --release --"
    fi
fi

info "Using forge: ${FORGE}"

# ─── Verify forge is functional ─────────────────────────────────────────
if ! ${FORGE} --version >/dev/null 2>&1; then
    echo ""
    fail "forge binary not functional: ${FORGE}"
    echo "  Build it first:  cargo build --release"
    exit 1
fi
FORGE_VERSION=$(${FORGE} --version 2>&1)
info "Forge version: ${FORGE_VERSION}"

# ─── Command availability checks ────────────────────────────────────────
# Some commands (diff, trace) may not be in older releases.
has_command() { ${FORGE} "$1" --help >/dev/null 2>&1; }

HAS_DIFF=false
HAS_TRACE=false
has_command diff  && HAS_DIFF=true
has_command trace && HAS_TRACE=true

if ! ${HAS_DIFF}; then
    skip "forge diff — not available in this build"
fi
if ! ${HAS_TRACE}; then
    skip "forge trace — not available in this build"
fi

# ─── Dependency checks ──────────────────────────────────────────────────
HAS_XMLLINT=false
if command -v xmllint >/dev/null 2>&1; then
    HAS_XMLLINT=true
fi

# ─── Test fixture ───────────────────────────────────────────────────────
FIXTURE="tests/fixtures/golden/small/input.md"
GOLDEN_DIR="tests/fixtures/golden/small"

if [[ ! -f "${FIXTURE}" ]]; then
    fail "Test fixture not found: ${FIXTURE}"
    exit 1
fi
info "Using fixture: ${FIXTURE}"

# ─── Working directory ─────────────────────────────────────────────────
TMPDIR=$(mktemp -d)
trap 'rm -rf "${TMPDIR}"' EXIT

info "Working directory: ${TMPDIR}"
echo ""

# ─── Counters ───────────────────────────────────────────────────────────
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

check() {
    local label="$1"
    shift
    TOTAL=$((TOTAL + 1))
    if "$@"; then
        PASSED=$((PASSED + 1))
        pass "${label}"
    else
        FAILED=$((FAILED + 1))
        fail "${label}"
    fi
}

check_skip() {
    local label="$1"
    shift
    TOTAL=$((TOTAL + 1))
    if "$@"; then
        PASSED=$((PASSED + 1))
        pass "${label}"
    else
        SKIPPED=$((SKIPPED + 1))
        skip "${label}"
    fi
}

# ═══════════════════════════════════════════════════════════════════════
# Step 1: Generate OSCAL models from test policy
# ═══════════════════════════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════════════"
echo "Step 1: Generate OSCAL models"
echo "═══════════════════════════════════════════════════════════"

# --- 1a: Catalog (JSON) ---
CATALOG_JSON="${TMPDIR}/catalog.json"
check "Generate catalog (JSON)" \
    ${FORGE} convert "${FIXTURE}" --strategy catalog --format json --output "${CATALOG_JSON}"

# --- 1b: Catalog (XML) ---
CATALOG_XML="${TMPDIR}/catalog.xml"
check "Generate catalog (XML)" \
    ${FORGE} convert "${FIXTURE}" --strategy catalog --format xml --output "${CATALOG_XML}"

# --- 1c: Profile (JSON) — must be generated before component (used as --source-profile) ---
# forge profile requires --include or --exclude. Extract control IDs from the catalog.
CONTROL_IDS=$(grep -o '"id" *: *"[A-Za-z0-9_-]*"' "${CATALOG_JSON}" \
    | sed 's/"id" *: *"//;s/"//' \
    | grep -v '_smt$\|_gdn$\|_obj$\|title-' \
    | head -20 \
    | tr '\n' ',' \
    | sed 's/,$//')

if [[ -z "${CONTROL_IDS}" ]]; then
    TOTAL=$((TOTAL + 1))
    FAILED=$((FAILED + 1))
    fail "Generate profile (JSON) — no control IDs found in catalog"
else
    PROFILE_JSON="${TMPDIR}/profile.json"
    check "Generate profile (JSON)" \
        ${FORGE} profile --catalog "${CATALOG_JSON}" --include "${CONTROL_IDS}" --format json --output "${PROFILE_JSON}"
fi

# --- 1d: Profile (XML) ---
if [[ -n "${CONTROL_IDS}" && -f "${PROFILE_JSON}" ]]; then
    PROFILE_XML="${TMPDIR}/profile.xml"
    check "Generate profile (XML)" \
        ${FORGE} profile --catalog "${CATALOG_JSON}" --include "${CONTROL_IDS}" --format xml --output "${PROFILE_XML}"
else
    TOTAL=$((TOTAL + 1))
    SKIPPED=$((SKIPPED + 1))
    skip "Generate profile (XML) — skipped (profile JSON generation failed)"
fi

# --- 1e: Component Definition (JSON) — requires --source-profile in v0.3.0+ ---
COMP_JSON="${TMPDIR}/component-definition.json"
if [[ -f "${PROFILE_JSON:-}" ]]; then
    check "Generate component-definition (JSON)" \
        ${FORGE} convert "${FIXTURE}" --strategy component --format json \
            --source-profile "${PROFILE_JSON}" --output "${COMP_JSON}"
else
    TOTAL=$((TOTAL + 1))
    FAILED=$((FAILED + 1))
    fail "Generate component-definition (JSON) — profile not available for --source-profile"
fi

# --- 1f: Component Definition (XML) ---
COMP_XML="${TMPDIR}/component-definition.xml"
if [[ -f "${PROFILE_JSON:-}" ]]; then
    check "Generate component-definition (XML)" \
        ${FORGE} convert "${FIXTURE}" --strategy component --format xml \
            --source-profile "${PROFILE_JSON}" --output "${COMP_XML}"
else
    TOTAL=$((TOTAL + 1))
    FAILED=$((FAILED + 1))
    fail "Generate component-definition (XML) — profile not available for --source-profile"
fi

# --- 1g: Assessment Plan (JSON) — secondary output of catalog + --import-ssp ---
# --import-ssp was added after v0.2.0; check if the flag is supported.
AP_JSON="${TMPDIR}/assessment-plan.json"
HAS_IMPORT_SSP=false
if ${FORGE} convert --help 2>&1 | grep -q '\-\-import-ssp'; then
    HAS_IMPORT_SSP=true
fi

if ${HAS_IMPORT_SSP}; then
    check "Generate assessment-plan (JSON)" \
        ${FORGE} convert "${FIXTURE}" --strategy catalog --format json \
            --output "${TMPDIR}/catalog-ap.json" --import-ssp "./system-ssp.json"

    # The assessment plan is written as a secondary output alongside the catalog.
    # Its filename is derived from the input stem: {input_stem}-assessment-plan.json
    AP_ACTUAL="${TMPDIR}/input-assessment-plan.json"
    if [[ -f "${AP_ACTUAL}" ]]; then
        cp "${AP_ACTUAL}" "${AP_JSON}"
        check "Assessment-plan file exists" test -s "${AP_JSON}"
    else
        # Fallback: check any *-assessment-plan.json in tmpdir
        AP_GLOB=$(find "${TMPDIR}" -name "*assessment-plan.json" 2>/dev/null | head -1)
        if [[ -n "${AP_GLOB}" && "${AP_GLOB}" != "${CATALOG_JSON}" ]]; then
            cp "${AP_GLOB}" "${AP_JSON}"
            check "Assessment-plan file exists" test -s "${AP_JSON}"
        else
            check "Assessment-plan file exists" false
        fi
    fi
else
    TOTAL=$((TOTAL + 2))
    SKIPPED=$((SKIPPED + 2))
    skip "Generate assessment-plan (JSON) — --import-ssp not available in this build"
    skip "Assessment-plan file exists — --import-ssp not available"
fi

# --- 1h: SSP — not exposed via CLI in current version ---
# build_ssp() exists in the library but has no CLI subcommand yet.
TOTAL=$((TOTAL + 1))
SKIPPED=$((SKIPPED + 1))
skip "Generate SSP (system-security-plan) — no CLI command yet (library-only: build_ssp)"

echo ""

# ═══════════════════════════════════════════════════════════════════════
# Step 2: Validate outputs against OSCAL schemas
# ═══════════════════════════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════════════"
echo "Step 2: Schema validation"
echo "═══════════════════════════════════════════════════════════"

# --- 2a: forge validate — catalog JSON ---
check "Validate catalog JSON (forge validate)" \
    ${FORGE} validate "${CATALOG_JSON}" --schema-type catalog --quiet

# --- 2b: forge validate — component JSON ---
check "Validate component JSON (forge validate)" \
    ${FORGE} validate "${COMP_JSON}" --schema-type component-definition --quiet

# --- 2c: xmllint — catalog XML against XSD ---
if ${HAS_XMLLINT}; then
    XSD_CATALOG="${REPO_ROOT}/tests/fixtures/xsd/oscal_catalog_schema.xsd"
    if [[ -f "${XSD_CATALOG}" ]]; then
        check "Validate catalog XML (xmllint --schema)" \
            xmllint --noout --schema "${XSD_CATALOG}" "${CATALOG_XML}" 2>/dev/null
    else
        check_skip "Validate catalog XML (xmllint) — XSD schema file not found" false
    fi
else
    check_skip "Validate catalog XML (xmllint — not installed)" false
fi

# --- 2d: xmllint — component XML against XSD ---
if ${HAS_XMLLINT}; then
    XSD_COMPONENT="${REPO_ROOT}/tests/fixtures/xsd/oscal_component_schema.xsd"
    if [[ -f "${XSD_COMPONENT}" ]]; then
        check "Validate component XML (xmllint --schema)" \
            xmllint --noout --schema "${XSD_COMPONENT}" "${COMP_XML}" 2>/dev/null
    else
        check_skip "Validate component XML (xmllint) — XSD schema file not found" false
    fi
else
    check_skip "Validate component XML (xmllint — not installed)" false
fi

# --- 2e: Structural validation — profile JSON has expected root key ---
if [[ -f "${PROFILE_JSON:-}" && -s "${PROFILE_JSON}" ]]; then
    check "Profile JSON has 'profile' root key" \
        python3 -c "
import json
with open('${PROFILE_JSON}') as f:
    data = json.load(f)
assert 'profile' in data, 'Missing profile root key'
"
else
    TOTAL=$((TOTAL + 1))
    SKIPPED=$((SKIPPED + 1))
    skip "Profile JSON root key — profile not generated"
fi

# --- 2f: Structural validation — assessment-plan JSON has expected root key ---
if [[ -f "${AP_JSON}" && -s "${AP_JSON}" ]]; then
    check "Assessment-plan JSON has 'assessment-plan' root key" \
        python3 -c "
import json
with open('${AP_JSON}') as f:
    data = json.load(f)
assert 'assessment-plan' in data, 'Missing assessment-plan root key'
" 2>/dev/null
else
    check_skip "Assessment-plan JSON root key — file not generated" false
fi

# --- 2g: forge validate — profile JSON (if supported in future) ---
if ${FORGE} validate --help 2>&1 | grep -q "profile"; then
    check "Validate profile JSON (forge validate)" \
        ${FORGE} validate "${PROFILE_JSON}" --schema-type profile --quiet
else
    check_skip "Validate profile JSON (forge validate — schema-type 'profile' not yet supported)" false
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════
# Step 3: Diff against golden fixtures
# ═══════════════════════════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════════════"
echo "Step 3: Golden file comparison (forge diff)"
echo "═══════════════════════════════════════════════════════════"

if ${HAS_DIFF}; then
    # --- 3a: Diff catalog against golden ---
    GOLDEN_CATALOG="${GOLDEN_DIR}/expected-catalog.json"
    if [[ -f "${GOLDEN_CATALOG}" ]]; then
        check "Diff catalog vs golden (zero differences)" \
            ${FORGE} diff "${GOLDEN_CATALOG}" "${CATALOG_JSON}"
    else
        check_skip "Diff catalog vs golden — golden file not found" false
    fi

    # --- 3b: Diff component against golden ---
    GOLDEN_COMP="${GOLDEN_DIR}/expected-component-definition.json"
    if [[ -f "${GOLDEN_COMP}" ]]; then
        check "Diff component vs golden (zero differences)" \
            ${FORGE} diff "${GOLDEN_COMP}" "${COMP_JSON}"
    else
        check_skip "Diff component vs golden — golden file not found" false
    fi
else
    TOTAL=$((TOTAL + 2))
    SKIPPED=$((SKIPPED + 2))
    skip "Diff catalog vs golden (forge diff not available)"
    skip "Diff component vs golden (forge diff not available)"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════
# Step 4: Traceability verification
# ═══════════════════════════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════════════"
echo "Step 4: Traceability report (forge trace)"
echo "═══════════════════════════════════════════════════════════"

if ${HAS_TRACE}; then
    TRACE_OUTPUT="${TMPDIR}/trace-report.txt"

    check "forge trace produces non-empty output" \
        ${FORGE} trace "${CATALOG_JSON}" --source "${FIXTURE}" --output "${TRACE_OUTPUT}"

    if [[ -f "${TRACE_OUTPUT}" ]]; then
        check "Trace report file is non-empty" \
            test -s "${TRACE_OUTPUT}"
    else
        # trace may write to stdout; re-run to capture
        TRACE_STDOUT=$(${FORGE} trace "${CATALOG_JSON}" --source "${FIXTURE}" 2>/dev/null || true)
        check "Trace stdout is non-empty" \
            test -n "${TRACE_STDOUT}"
    fi
else
    TOTAL=$((TOTAL + 2))
    SKIPPED=$((SKIPPED + 2))
    skip "forge trace produces non-empty output (command not available)"
    skip "Trace report file is non-empty (command not available)"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════════════"
echo "Results: ${PASSED}/${TOTAL} passed, ${FAILED} failed, ${SKIPPED} skipped"
echo "═══════════════════════════════════════════════════════════"

if [[ ${FAILED} -gt 0 ]]; then
    echo -e "${RED}Integration test FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}Integration test PASSED${NC}"
    exit 0
fi
