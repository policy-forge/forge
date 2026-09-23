#!/usr/bin/env bash
#
# Fetch third-party OSCAL catalogs into a gitignored scratch folder.
# Files are transient working copies: move them out of the destination
# folder once downloaded; re-run the script to fetch them again.
#
# Usage:
#   scripts/fetch-oscal-catalogs.sh [--source registry|curated|all] [--clean] [--dest DIR]
#
# Sources:
#   registry  every catalog listed at https://registry.oscal.io (deduped)
#   curated   hand-picked first-party catalogs (NIST, FedRAMP, CIS, CMS, ACSC, ...)
#   all       both of the above (default)

set -Eeuo pipefail
trap 'status=$?; echo "[fetch-oscal] FAILED (exit ${status}) at line ${LINENO}: ${BASH_COMMAND}" >&2; exit "${status}"' ERR

SOURCE="${BASH_SOURCE[0]}"
while [[ -h "${SOURCE}" ]]; do
    SOURCE_DIR="$(cd -P "$(dirname "${SOURCE}")" && pwd)"
    SOURCE="$(readlink "${SOURCE}")"
    [[ "${SOURCE}" != /* ]] && SOURCE="${SOURCE_DIR}/${SOURCE}"
done
SCRIPT_DIR="$(cd -P "$(dirname "${SOURCE}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

WHICH="all"
CLEAN=0
DEST="${REPO_ROOT}/.oscal-catalogs"
REGISTRY_API="https://registry.oscal.io/api/v1/catalogs"

usage() {
    sed -n '2,14p' "${SOURCE}" | sed 's/^# \{0,1\}//'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --source) WHICH="${2:?--source requires a value}"; shift 2 ;;
        --clean)  CLEAN=1; shift ;;
        --dest)   DEST="${2:?--dest requires a value}"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "[fetch-oscal] unknown option: $1" >&2; exit 2 ;;
    esac
done

case "${WHICH}" in
    registry|curated|all) ;;
    *) echo "[fetch-oscal] --source must be registry, curated, or all (got: ${WHICH})" >&2; exit 2 ;;
esac

log() { echo "[fetch-oscal] $*"; }

CURL=(curl -fsSL --retry 3 --retry-delay 2 --connect-timeout 15 -m 600)

mkdir -p "${DEST}"
if [[ "${CLEAN}" -eq 1 ]]; then
    log "cleaning ${DEST}"
    find "${DEST}" -mindepth 1 -delete
fi

FAILURES=()
MANIFEST="${DEST}/_manifest.tsv"
: > "${MANIFEST}"

# fetch <relative-output-path> <url> [manifest-title]
fetch() {
    local out="${DEST}/$1" url="$2" title="${3:-$1}"
    local dir
    dir="$(dirname "${out}")"
    mkdir -p "${dir}"
    if "${CURL[@]}" -o "${out}" "${url}"; then
        printf '%s\t%s\t%s\n' "$1" "${url}" "${title}" >> "${MANIFEST}"
        log "ok  $1"
    else
        rm -f "${out}"
        FAILURES+=("$1 (${url})")
        log "ERR $1  <- ${url}"
    fi
}

# validate_json <file>: reject HTML error pages / truncated downloads served as 200s
validate_json() {
    python3 - "$1" <<'PY'
import json, sys
with open(sys.argv[1], "rb") as f:
    data = json.load(f)
if not isinstance(data, dict) or "catalog" not in data:
    sys.exit("not an OSCAL catalog document")
PY
}

fetch_registry() {
    log "listing ${REGISTRY_API}"
    local listing
    listing="$(python3 - "${REGISTRY_API}" <<'PY'
import json, re, sys, urllib.request

with urllib.request.urlopen(sys.argv[1], timeout=60) as r:
    data = json.load(r)

seen = set()
used_names = {}

def slug(s):
    s = re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")
    return s[:60] or "catalog"

for c in data:
    title = (c.get("title") or "untitled").replace("\t", " ")
    if "for demonstration" in title.lower():
        continue  # registry sample junk
    version = c.get("document-version") or ""
    key = (title, version)
    if key in seen:
        continue  # same catalog republished by multiple owners
    seen.add(key)
    version = version if re.fullmatch(r"[A-Za-z0-9._+-]{1,24}", version) else ""
    base = slug(title) + (("-" + slug(version)) if version else "")
    n = used_names.get(base, 0) + 1
    used_names[base] = n
    name = base if n == 1 else f"{base}-{n}"
    print("\t".join((c["self"], name + ".json", title)))
PY
)"
    local count
    count="$(printf '%s\n' "${listing}" | grep -c . || true)"
    log "downloading ${count} registry catalogs"
    local path name title
    while IFS=$'\t' read -r path name title; do
        [[ -z "${path}" ]] && continue
        local out="${name}"
        fetch "${out}" "https://registry.oscal.io${path}" "${title}"
        if [[ -f "${DEST}/${out}" ]]; then
            if ! validate_json "${DEST}/${out}" 2>/dev/null; then
                rm -f "${DEST}/${out}"
                FAILURES+=("${out} (invalid JSON from https://registry.oscal.io${path})")
                log "ERR ${out} is not a valid OSCAL catalog; removed"
            fi
        fi
        sleep 0.3  # be polite to the registry
    done <<< "${listing}"
}

fetch_curated() {
    local base_nist="https://raw.githubusercontent.com/usnistgov/oscal-content/main/nist.gov"
    local base_fedramp="https://raw.githubusercontent.com/OSCAL-Foundation/fedramp-resources/main/baselines/rev5/json"
    local base_cac="https://raw.githubusercontent.com/ComplianceAsCode/oscal-content/main/catalogs"
    local base_ars="https://raw.githubusercontent.com/CMSgov/ars-machine-readable/main"
    local base_ism="https://raw.githubusercontent.com/AustralianCyberSecurityCentre/ism-oscal/main"

    local entries=(
        # NIST SP 800-53 Rev 5: catalog + security/privacy baselines
        "nist/SP800-53_rev5_catalog.json|${base_nist}/SP800-53/rev5/json/NIST_SP-800-53_rev5_catalog.json|NIST SP 800-53 Rev 5 catalog"
        "nist/SP800-53_rev5_LOW-baseline_profile.json|${base_nist}/SP800-53/rev5/json/NIST_SP-800-53_rev5_LOW-baseline_profile.json|NIST SP 800-53 Rev 5 LOW baseline"
        "nist/SP800-53_rev5_MODERATE-baseline_profile.json|${base_nist}/SP800-53/rev5/json/NIST_SP-800-53_rev5_MODERATE-baseline_profile.json|NIST SP 800-53 Rev 5 MODERATE baseline"
        "nist/SP800-53_rev5_HIGH-baseline_profile.json|${base_nist}/SP800-53/rev5/json/NIST_SP-800-53_rev5_HIGH-baseline_profile.json|NIST SP 800-53 Rev 5 HIGH baseline"
        "nist/SP800-53_rev5_PRIVACY-baseline_profile.json|${base_nist}/SP800-53/rev5/json/NIST_SP-800-53_rev5_PRIVACY-baseline_profile.json|NIST SP 800-53 Rev 5 PRIVACY baseline"
        # NIST SP 800-53 Rev 4
        "nist/SP800-53_rev4_catalog.json|${base_nist}/SP800-53/rev4/json/NIST_SP-800-53_rev4_catalog.json|NIST SP 800-53 Rev 4 catalog"
        "nist/SP800-53_rev4_MODERATE-baseline_profile.json|${base_nist}/SP800-53/rev4/json/NIST_SP-800-53_rev4_MODERATE-baseline_profile.json|NIST SP 800-53 Rev 4 MODERATE baseline"
        # Other NIST publications
        "nist/CSF_v2.0_catalog.json|${base_nist}/CSF/v2.0/json/NIST_CSF_v2.0_catalog.json|NIST Cybersecurity Framework 2.0"
        "nist/SP800-171_rev3_catalog.json|${base_nist}/SP800-171/rev3/json/NIST_SP800-171_rev3_catalog.json|NIST SP 800-171 Rev 3"
        "nist/SP800-172_catalog.json|${base_nist}/SP800-172/rev3/json/NIST_SP800-172_rev3_catalog.json|NIST SP 800-172"
        "nist/SP800-218_SSDF_v1.1_catalog.json|${base_nist}/SP800-218/ver1/json/NIST_SP800-218_ver1_catalog.json|NIST SP 800-218 SSDF v1.1"
        # FedRAMP Rev 5 baselines (GSA/fedramp-automation was removed; OSCAL-Foundation continues it)
        "fedramp/rev5_LOW-baseline_profile.json|${base_fedramp}/FedRAMP_rev5_LOW-baseline_profile.json|FedRAMP Rev 5 LOW baseline"
        "fedramp/rev5_MODERATE-baseline_profile.json|${base_fedramp}/FedRAMP_rev5_MODERATE-baseline_profile.json|FedRAMP Rev 5 MODERATE baseline"
        "fedramp/rev5_HIGH-baseline_profile.json|${base_fedramp}/FedRAMP_rev5_HIGH-baseline_profile.json|FedRAMP Rev 5 HIGH baseline"
        "fedramp/rev5_LI-SaaS-baseline_profile.json|${base_fedramp}/FedRAMP_rev5_LI-SaaS-baseline_profile.json|FedRAMP Rev 5 LI-SaaS baseline"
        # CIS (official)
        "cis/cis-controls-v8_OSCAL-1.0.xml|https://raw.githubusercontent.com/CISecurity/CISControls_OSCAL/main/src/catalogs/xml/cis-controls-v8_OSCAL-1.0.xml|CIS Controls v8"
        # CMS Acceptable Risk Safeguards
        "cms-ars/CMS_ARS_5_0_catalog.json|${base_ars}/5.0/oscal/json/CMS_ARS_5_0_catalog.json|CMS ARS 5.0 catalog"
        "cms-ars/cms_ars_50_low.json|${base_ars}/5.0/oscal/json/cms_ars_50_low.json|CMS ARS 5.0 LOW baseline"
        "cms-ars/cms_ars_50_moderate.json|${base_ars}/5.0/oscal/json/cms_ars_50_moderate.json|CMS ARS 5.0 MODERATE baseline"
        "cms-ars/cms_ars_50_high.json|${base_ars}/5.0/oscal/json/cms_ars_50_high.json|CMS ARS 5.0 HIGH baseline"
        "cms-ars/CMS_ARS_3_1_catalog.json|${base_ars}/3.1/oscal/json/CMS_ARS_3_1_catalog.json|CMS ARS 3.1 catalog"
        # Community conversions (ComplianceAsCode / Red Hat)
        "community/pcidss_4_catalog.json|${base_cac}/pcidss_4/catalog.json|PCI DSS v4 (community conversion)"
        "community/hipaa_catalog.json|${base_cac}/hipaa/catalog.json|HIPAA Security Rule (community conversion)"
        "community/e8_catalog.json|${base_cac}/e8/catalog.json|Essential Eight (community conversion)"
        # ACSC ISM + Essential Eight baselines (official mirror)
        "acsc-ism/ISM_NON_CLASSIFIED-baseline_profile.json|${base_ism}/ISM_NON_CLASSIFIED-baseline_profile.json|ACSC ISM NON-CLASSIFIED baseline"
        "acsc-ism/ISM_OFFICIAL_SENSITIVE-baseline_profile.json|${base_ism}/ISM_OFFICIAL_SENSITIVE-baseline_profile.json|ACSC ISM OFFICIAL:SENSITIVE baseline"
        "acsc-ism/ISM_PROTECTED-baseline_profile.json|${base_ism}/ISM_PROTECTED-baseline_profile.json|ACSC ISM PROTECTED baseline"
        "acsc-ism/ISM_E8_ML1-baseline_profile.json|${base_ism}/ISM_E8_ML1-baseline_profile.json|ACSC Essential Eight ML1 baseline"
        "acsc-ism/ISM_E8_ML2-baseline_profile.json|${base_ism}/ISM_E8_ML2-baseline_profile.json|ACSC Essential Eight ML2 baseline"
        "acsc-ism/ISM_E8_ML3-baseline_profile.json|${base_ism}/ISM_E8_ML3-baseline_profile.json|ACSC Essential Eight ML3 baseline"
    )

    local entry name url title rest
    log "downloading ${#entries[@]} curated catalogs"
    for entry in "${entries[@]}"; do
        name="${entry%%|*}"
        rest="${entry#*|}"
        url="${rest%%|*}"
        title="${rest#*|}"
        fetch "${name}" "${url}" "${title}"
    done
}

if [[ "${WHICH}" == "registry" || "${WHICH}" == "all" ]]; then
    fetch_registry
fi
if [[ "${WHICH}" == "curated" || "${WHICH}" == "all" ]]; then
    fetch_curated
fi

log "------------------------------------------------------------"
log "downloaded: $(grep -c . "${MANIFEST}" || true) file(s) into ${DEST}"
log "manifest:   ${MANIFEST}"
if [[ "${#FAILURES[@]}" -gt 0 ]]; then
    log "${#FAILURES[@]} failure(s):"
    for f in "${FAILURES[@]}"; do
        log "  - ${f}"
    done
    exit 1
fi
log "done"
