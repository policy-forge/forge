// @ts-check
// The document contains no project data. All reads use the supported local API.
const element = (id) => document.getElementById(id);
// Only a server-selected supported major can own this page's local requests.
const declaredApiMajor = document.querySelector('meta[name="forge-api-major"]')?.getAttribute("content");
const apiMajor = declaredApiMajor === "1" ? 1 : declaredApiMajor === "2" ? 2 : null;
const apiContractVersion = document.querySelector('meta[name="forge-api-contract-version"]')?.getAttribute("content");
const apiPrefix = `/api/v${apiMajor}`;
let capability = "";
let activeView = "Overview";
let stopped = false;
let unlockPending = false;
let previewGeneration = 0;
let previewClosePending = null;
let pending = 0;
let readOnly = true;
let dirty = false;
let nextRequestAt = 0;
let viewFilters = {};
// The server retains at most 256 operations per session.
const operationRows = new Map();
// Bound unacknowledged or direct-preview requests independently from server operations.
const requestRows = new Map();
// Disconnected pagers cannot retain their view-resume callbacks.
const retainedPagers = new WeakMap();
// Installed metadata callbacks retire local disclosure ownership on every view epoch.
const bundlePanels = new WeakMap();
// Only committed metadata/source downloads retain locally revocable Blob handles.
const metadataDownloadURLs = new Set();
// Public outcome IDs survive view changes; no receipt or credential is persisted.
const sourceRestoreRows = new Map();
// Captured S3 pane reads retire independently when a parent selection/date changes.
const inspectionReaders = new WeakMap();
const titles = ["Overview", "Review Queue", "Framework Scope", "Mappings", "Policies & Artifacts", "Trace & Reports", ...(inspectionSupported() ? ["Lifecycle & Impact"] : [])];

function node(tag, text, className) {
  const result = document.createElement(tag);
  if (text !== undefined) result.textContent = String(text);
  if (className) result.className = className;
  return result;
}

function showError(error) {
  const modal = document.querySelector("dialog[open]");
  let box = modal?.querySelector("[role=alert]");
  if(modal && !box) {box=node("div");box.setAttribute("role","alert");box.tabIndex=-1;modal.prepend(box);}
  box ||= element("error");
  box.textContent = error instanceof Error ? error.message : "The operation could not be completed.";
  box.hidden = false;
  box.focus();
}

/** Send declared API1/supported API2 calls; preserve exact metadata/source raw bodies and pacing fences. */
async function api(path, method = "GET", body, key, rawBody, rawIsCurrent) {
  if (![1, 2].includes(apiMajor) || apiMajor === 2 && !["2.0.0", "2.1.0", "2.2.0", "2.3.0"].includes(apiContractVersion)) throw new Error("This page requires matching supported workspace API assets. Relaunch with the selected API major.");
  if (rawBody !== undefined && (method !== "POST" || !(rawBody instanceof Blob) || rawBody.size > 1024 * 1024 || !(path === "/project/bundle-verifications" && !key || path === "/project/bundle-imports" && bundleEffectsSupported() && !!key || path === "/project/source-bundle-imports" && sourceBundleEffectsSupported() && !!key))) throw new Error("Unsupported raw metadata request.");
  const now=performance.now();const reserved=Math.max(now,nextRequestAt);nextRequestAt=reserved+60;
  if(reserved>now)await new Promise(resolve=>setTimeout(resolve,reserved-now));
  if (stopped) throw new Error("This workspace has stopped. Relaunch it from the terminal.");
  if (rawIsCurrent && !rawIsCurrent()) throw new Error("The metadata comparison was superseded before sending.");
  const headers = { "Accept": "application/json" };
  if (capability) headers["Authorization"] = `Bearer ${capability}`;
  if (method !== "GET") headers["Content-Type"] = "application/json";
  if (key) headers["Idempotency-Key"] = key;
  let response;
  try {
    response = await fetch(`${apiPrefix}${path}`, {method, headers, body: method === "GET" ? undefined : rawBody ?? JSON.stringify(body ?? {}), cache: "no-store", credentials: "omit", redirect: "error", referrerPolicy: "no-referrer"});
  } catch {
    throw new Error("The local workspace is unavailable. Check its terminal before retrying.");
  }
  const value = await response.json();
  if (!response.ok) {const error=new Error(`${value.code}: ${value.message} Retryable: ${value.retryable}.${value.field?` Field: ${value.field}.`:""}${value.resource_version?` Current version: ${value.resource_version}.`:""}`);error.details=value;throw error;}
  return value;
}

async function collection(path) {
  let result = [];
  let cursor = null;
  let total = 0;
  do {
    const page = await api(`${path}${path.includes("?") ? "&" : "?"}page_size=200${cursor ? `&cursor=${encodeURIComponent(cursor)}` : ""}`);
    result.push(...page.page.items);
    total = page.page.total_matching;
    cursor = page.page.next_cursor;
    if(cursor) await new Promise(resolve=>setTimeout(resolve,100));
    if (result.length > 10000) throw new Error("This view exceeds the supported display bound. Narrow the review scope.");
  } while (cursor);
  if (result.length !== total) throw new Error("The collection changed. Refresh this view.");
  return result;
}

function table(caption, columns, rows) {
  const wrap = node("div", undefined, "table-wrap");
  wrap.tabIndex = 0;
  wrap.setAttribute("role", "region");
  wrap.setAttribute("aria-label", caption);
  const grid = node("table");
  grid.append(node("caption", caption));
  const head = node("thead");
  const heading = node("tr");
  for (const [label] of columns) { const cell = node("th", label); cell.scope = "col"; heading.append(cell); }
  head.append(heading);
  const body = node("tbody");
  for (const row of rows) { const line = node("tr"); for (const [,key] of columns) line.append(node("td", row[key] ?? "—")); body.append(line); }
  grid.append(head, body); wrap.append(grid); return wrap;
}

/** Validate bounded list metadata before publishing rows or a reconciled count. */
function checkedPage(response, size) {
  const page = response?.page;
  if (typeof response?.resource_version !== "string" || response.resource_version.length < 8 || response.resource_version.length > 128 ||
      !Array.isArray(page?.items) || page.items.length > size || !Number.isSafeInteger(page.total_matching) || page.total_matching < page.items.length ||
      !(page.next_cursor === null || (typeof page.next_cursor === "string" && page.next_cursor.length > 0 && page.next_cursor.length <= 512)) ||
      (page.next_cursor && !page.items.length)) throw new Error("The list returned unsupported page metadata. Retry this read.");
  return page;
}

/** Keep verified rows and allow a retained live pager to recover after a dismissed write refresh. */
async function pagedTable(path, caption, columns, filters = []) {
  let owner = pending;
  const section = node("section"); const form = node("form"); const display = node("div"); const controls = node("div");
  section.setAttribute("data-paged-table", "");
  display.tabIndex = -1; display.setAttribute("role", "region"); display.setAttribute("aria-label", caption); display.setAttribute("data-page-results", "");
  const status = node("p"); status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite"); status.setAttribute("aria-atomic", "true"); status.setAttribute("data-page-status", "");
  const error = node("div"); error.hidden = true; error.tabIndex = -1; error.setAttribute("role", "alert"); error.setAttribute("data-page-error", "");
  const choices = filters.map(([key, label, values]) => {
    const select = fieldInput(form, label, "select", [["", "All"], ...values.map(value => [value, value.replaceAll("-", " ")])]);
    select.required = false; if (viewFilters[key]) select.value = viewFilters[key]; return [key, select];
  });
  let cursors = [null]; let index = 0; let applied = Object.fromEntries(choices.map(([key, select]) => [key, select.value]));
  let generation = 0; let busy = false; let summary = ""; let nextCursor = null; let lastRequest;
  const actions = [];
  /** Rebind only a retained installed pager; older responses and their finally handlers stay obsolete. */
  function resumePage() {
    if (stopped || !section.isConnected || element("view").inert || owner === pending) return;
    owner = pending; generation++; busy = false;
    display.setAttribute("aria-busy", "false"); actions.forEach(control => control.setAttribute("aria-disabled", "false"));
    if (summary) status.textContent = `Previous results retained after the refresh was cancelled: ${summary}`;
  }
  retainedPagers.set(section, resumePage);
  /** Keep the initiating control focusable while preventing repeated pending reads. */
  const pageAction = (label, action) => {
    const control = node("button", label); control.type = "button"; actions.push(control);
    control.setAttribute("aria-disabled", "false");
    control.addEventListener("click", async () => { if (!busy && control.getAttribute("aria-disabled") !== "true") await action(); });
    return control;
  };
  const previous = pageAction("Previous page", () => load(index - 1, cursors, applied, true)); previous.hidden = true;
  const next = pageAction("Next page", () => { const staged = cursors.slice(); staged[index + 1] = nextCursor; return load(index + 1, staged, applied, true); }); next.hidden = true;
  const retry = pageAction("Retry page", () => load(lastRequest.index, lastRequest.cursors, lastRequest.filters, true)); retry.hidden = true;
  const restart = pageAction("Restart from first page", () => load(0, [null], lastRequest.filters, true)); restart.hidden = true;
  /** Stage verified pages; release only this latest local read even if its view fence became obsolete. */
  async function load(targetIndex, targetCursors, targetFilters, focusResults = false) {
    const sequence = ++generation; const attached = section.isConnected;
    /** Only the latest read under this pager's current view owner may publish results or focus. */
    const current = () => !stopped && owner === pending && sequence === generation && (!attached || section.isConnected);
    lastRequest = { index: targetIndex, cursors: targetCursors.slice(), filters: { ...targetFilters } };
    busy = true; actions.forEach(control => control.setAttribute("aria-disabled", "true"));
    display.setAttribute("aria-busy", "true"); error.hidden = true;
    status.textContent = `${caption}: Loading results…${summary ? ` Previous results: ${summary}` : ""}`;
    try {
      const query = new URLSearchParams({ page_size: "50" });
      for (const [key, value] of Object.entries(targetFilters)) if (value) query.set(key, value);
      if (targetCursors[targetIndex]) query.set("cursor", targetCursors[targetIndex]);
      const filtered = Object.values(targetFilters).some(Boolean);
      const response = await api(`${path}?${query}`); const page = checkedPage(response, 50);
      let total = page.total_matching;
      if (filtered) {
        const unfiltered = await api(`${path}?page_size=1`); const denominator = checkedPage(unfiltered, 1);
        if (response.resource_version !== unfiltered.resource_version || denominator.total_matching < page.total_matching) {
          const conflict = new Error("The list changed between filtered and total reads. Restart from its first page.");
          conflict.details = { code: "version-conflict", retryable: true }; throw conflict;
        }
        total = denominator.total_matching;
      }
      if (!current()) return;
      index = targetIndex; cursors = targetCursors.slice(); applied = { ...targetFilters }; nextCursor = page.next_cursor;
      display.replaceChildren(page.items.length ? table(caption, columns, page.items) : node("p", "No matching items.", "empty"), evidenceList(page.items));
      summary = `${page.total_matching} matching items of ${total} total · Page ${index + 1}.`;
      status.textContent = summary; retry.hidden = true; restart.hidden = true;
      if (focusResults && section.isConnected) display.focus();
      previous.hidden = index === 0; next.hidden = !nextCursor;
    } catch (failure) {
      if (!current()) return;
      error.textContent = `${failure instanceof Error ? failure.message : "The list could not be read."}${summary ? " Previous results are retained; they do not reflect this failed read." : " No results could be loaded."}`;
      error.hidden = false; status.textContent = `${caption}: Results could not be loaded.${summary ? ` Previous results: ${summary}` : ""}`;
      retry.hidden = false; restart.hidden = failure.details?.code !== "version-conflict";
      if (section.isConnected) error.focus();
    } finally {
      if (!stopped && sequence === generation) { busy = false; display.setAttribute("aria-busy", "false"); actions.forEach(control => control.setAttribute("aria-disabled", "false")); }
    }
  }
  form.addEventListener("submit", event => event.preventDefault());
  if (choices.length) form.append(pageAction("Apply filters", () => load(0, [null], Object.fromEntries(choices.map(([key, select]) => [key, select.value])), true)));
  controls.append(previous, next, retry, restart); section.append(form, status, error, display, controls);
  await load(0, cursors, applied); return section;
}

/** Install only an owned live view, including gated S3 reads; dismissed write previews cannot publish a late refresh. */
async function renderView(isCurrent = () => true) {
  if (stopped || !isCurrent()) return false;
  invalidateBundlePanels();
  const previousStatus = element("status").textContent;
  const sequence = ++pending;
  element("error").hidden = true;
  element("status").textContent = "Loading project state…";
  element("refresh").disabled = true;
  element("view").inert = true;
  const fragment = document.createDocumentFragment();
  let pageError;
  try {
    if (activeView === "Overview") {
      const summary = await api("/project/summary");
      fragment.append(node("p", summary.project_label, "eyebrow"));
      const cards = node("div", undefined, "cards");
      for (const [label,count,destination,filters] of [
        ["Registered resources",summary.resource_counts.total,"Policies & Artifacts",{}],
        ["Open review items",summary.review_counts.total_open,"Review Queue",{}],
        ["Invalid inputs",summary.resource_counts.invalid,"Policies & Artifacts",{validation_state:"invalid"}],
        ["Stale inputs",summary.resource_counts.stale,"Policies & Artifacts",{stale:"true"}]
      ]) {
        const card=button("",()=>navigate(destination,filters,destination === "Review Queue"));card.className="card";
        card.append(node("strong",count),node("span",label));cards.append(card);
      }
      fragment.append(cards);
      const config = await api("/project/config-status");
      fragment.append(node("p", `Project configuration: ${config.present ? (config.valid ? "valid" : "invalid") : "not present"}.`));
      if (!summary.workspace_index_present) fragment.append(node("p", "No resources are registered yet. Register project files to begin reviewing scope and mappings.", "empty"));
      else fragment.append(node("p", `Next action: ${summary.next_action.replaceAll("-", " ")}.`));
      fragment.append(node("p", "These views show supplied records and drafting work. They do not establish implementation, effectiveness, approval, or compliance.", "muted"));
    } else if (activeView === "Policies & Artifacts") {
      const rows = await collection("/resources");
      const visible=rows.filter(row=>(!viewFilters.validation_state||row.validation_state===viewFilters.validation_state)&&(!viewFilters.stale||String(row.stale)===viewFilters.stale));
      if(Object.keys(viewFilters).length)fragment.append(node("p",`Showing ${visible.length} matching resources of ${rows.length} registered.`),button("Show all resources",()=>navigate("Policies & Artifacts")));
      fragment.append(visible.length ? table("Registered project files", [["Key","key"],["Role","role"],["Path","path"],["Validation","validation_state"],...(apiMajor === 2 ? [["Admission profile","validation_profile"]] : [])], visible) : node("p", "No matching registered files. Unregistered files are never scanned.", "empty"));
      fragment.append(evidenceList(visible.map(row=>({...row,label:row.key,provenance_ref:row.resource_id}))));
      fragment.append(resourceActions(rows));
    } else if (activeView === "Review Queue") {
      fragment.append(await pagedTable("/review-queue/items","Items requiring human review",[["Reason","reason_code"],["Control","control_id"],["Summary","summary"]],[
        ["reason_code","Reason",["reviewed-no-positive-relationship","no-reviewed-mapping","deferred-scope-decision","scope-decision-required","invalid-resource","stale-input","external-conflict"]]
      ]));
    } else if (activeView === "Framework Scope") {
      try {fragment.append(await pagedTable("/applicability/controls","Framework control inventory",[["Control","control_id"],["Decision","decision_state"],["Classification","classification"],["Review reason","review_reason"]],[
        ["classification","Classification",["applicable-mapped","applicable-reviewed-no-relationship","applicable-unmapped","not-applicable","deferred","under-review"]]
      ]));} catch(error) {fragment.append(node("p",error.message));}
      fragment.append(await draftEditor("applicability"));
    } else if (activeView === "Mappings") {
      try {fragment.append(await pagedTable("/mapping/subjects","Policy and framework subjects",[["Side","side"],["Subject","subject_id"],["Label","label"]],[["side","Side",["policy","framework"]]]));}
      catch(error) {fragment.append(node("p",error.message));}
      fragment.append(await draftEditor("mapping"));
    } else if (activeView === "Lifecycle & Impact") {
      fragment.append(await lifecycleImpactView());
    } else {
      const resources = await collection("/resources");
      fragment.append(node("p", "Select a registered resource to inspect its source and decision references."));
      for (const resource of resources) fragment.append(button(`Trace ${resource.key}`, () => showProvenance(resource.resource_id)));
      if (!readOnly) fragment.append(exportForm());
      fragment.append(metadataBundlePanel());
      if (bundleEffectsSupported()) fragment.append(bundleEffectsPanel());
      if (sourceBundleEffectsSupported()) fragment.append(sourceBundleEffectsPanel());
    }
    if (sequence !== pending || stopped || !isCurrent()) return false;
    element("view").replaceChildren(fragment);
    element("view-title").textContent = activeView;
    pageError = element("view").querySelector("[data-page-error]");
    if (pageError?.hidden) pageError = undefined;
    element("status").textContent = pageError ? "The table could not be loaded." : "Project state loaded.";
    return !pageError;
  } catch (error) { if (sequence === pending && !stopped && isCurrent()) { element("view").replaceChildren(); showError(error); element("status").textContent = "The view could not be loaded."; } return false; }
  finally { if (sequence === pending && !stopped) {
    element("refresh").disabled = false; element("view").inert = false;
    if (isCurrent()) pageError?.focus();
    else {
      if (element("status").textContent === "Loading project state…") element("status").textContent = previousStatus;
      for (const section of element("view").querySelectorAll("[data-paged-table]")) retainedPagers.get(section)?.();
    }
  } }
}

/** Keep S3 reads available across their consumed additive2.1/2.2/2.3 contracts. */
function inspectionSupported() {
  return apiMajor === 2 && ["2.1.0", "2.2.0", "2.3.0"].includes(apiContractVersion);
}

/** Validate only the exact typed identifiers/capture hashes consumed by S3 read controls. */
function inspectionIdentity(value, hash = false) {
  return typeof value === "string" && (hash ? /^[0-9a-f]{64}$/.test(value) : /^res_[0-9a-z]{12,80}$/.test(value));
}

/** Require consumed nullable/metadata fields explicitly rather than treating omission as null. */
function inspectionRequired(value, fields) {
  if (!value || typeof value !== "object" || Array.isArray(value) || fields.some(key => !Object.hasOwn(value, key))) throw new Error("The read omitted required captured metadata. Retry this read.");
}

/** Preserve every subject/hash pair in a bounded response using one escaped text region, not unbounded DOM rows. */
function inspectionSubjects(caption, values) {
  if (!Array.isArray(values) || values.length > 100000) throw new Error("The read returned unsupported subject fingerprints.");
  if (values.some(value => typeof value?.id !== "string" || !inspectionIdentity(value.sha256, true))) throw new Error("The read returned unsupported subject fingerprint types.");
  const section = node("section"); section.append(node("h4", `${caption}: ${values.length} fingerprints`), node("pre", values.map(value => `${inspectionValue(value.id)} · ${inspectionValue(value.sha256)}`).join("\n") || "None")); return section;
}

/** Display declared scalar metadata without serializing unknown objects or source-bearing fields. */
function inspectionValue(value) {
  return typeof value === "string" || typeof value === "number" || typeof value === "boolean" ? String(value) : "—";
}

/** Display only explicit scalar identity tokens from a declared metadata array. */
function inspectionTokens(values) {
  return Array.isArray(values) ? values.map(inspectionValue).join(", ") || "None declared" : "—";
}

/** Build escaped labelled metadata; callers select every visible field explicitly. */
function inspectionMetadata(caption, entries) {
  const section = node("section"); section.append(node("h3", caption));
  const list = node("dl");
  for (const [label, value] of entries) list.append(node("dt", label), node("dd", inspectionValue(value)));
  section.append(list); return section;
}

/** Validate a complete unfiltered ChangeSummary before rendering distinct integer scopes. */
function inspectionSummary(summary) {
  const labels = [["old_controls", "Old controls"], ["new_controls", "New controls"], ["added", "Added"], ["removed", "Removed"],
    ["content_changed", "Content changed"], ["identity_migrated", "Identity migrated"], ["unchanged", "Unchanged"],
    ["findings", "Findings"], ["blocking", "Blocking"], ["review_required", "Review required"], ["informational", "Informational"],
    ["dispositioned_resolved", "Resolved dispositions"], ["dispositioned_accepted_risk", "Accepted-risk dispositions"],
    ["dispositioned_still_open", "Still-open dispositions"], ["undispositioned", "Undispositioned"]];
  if (!summary || labels.some(([key]) => !Number.isSafeInteger(summary[key]) || summary[key] < 0 || summary[key] > 100000)) throw new Error("The comparison returned unsupported full summary metadata. Retry this capture.");
  return table("Complete unfiltered comparison summary", [["Measure", "measure"], ["Count", "count"]], labels.map(([key, measure]) => ({measure, count: summary[key]})));
}

/** Show only redacted captured registration provenance, never source text or parser diagnostics. */
function inspectionProvenance(rows) {
  if (!Array.isArray(rows) || rows.length > 1000 || rows.some(row => !inspectionIdentity(row?.resource_id) || !inspectionIdentity(row.sha256, true) || !Number.isSafeInteger(row.size_bytes) || row.size_bytes < 0 || row.size_bytes > 10485760)) throw new Error("The read returned unsupported provenance metadata.");
  return table("Captured registered provenance", [["Resource", "resource_id"], ["Role", "role"], ["SHA-256", "sha256"], ["Bytes", "size_bytes"], ["Admission", "validation_state"]], rows);
}

/** Show declared/computed fingerprint identities without href, title, rationale or party prose. */
function inspectionPair(label, value) {
  if (value !== null) {
    inspectionRequired(value, ["resource_id", "resource_type", "raw_sha256", "root_uuid", "document_version", "oscal_version", "resolved_catalog_sha256", "resolved_catalog_resource_id"]);
    if (!inspectionIdentity(value.resource_id) || !inspectionIdentity(value.raw_sha256, true) || !["catalog", "profile"].includes(value.resource_type) ||
        value.resource_type === "catalog" && (value.resolved_catalog_sha256 !== null || value.resolved_catalog_resource_id !== null) ||
        value.resource_type === "profile" && (!inspectionIdentity(value.resolved_catalog_sha256, true) || !inspectionIdentity(value.resolved_catalog_resource_id))) throw new Error("The comparison returned unsupported fingerprint identity types.");
  }
  return inspectionMetadata(label, [["Resource", value?.resource_id], ["Type", value?.resource_type], ["Raw SHA-256", value?.raw_sha256],
    ["Root UUID", value?.root_uuid], ["Document version", value?.document_version], ["OSCAL version", value?.oscal_version],
    ["Resolved catalog SHA-256", value?.resolved_catalog_sha256], ["Resolved catalog resource", value?.resolved_catalog_resource_id]]);
}

/** Render only lifecycle fingerprint hashes and captured identity metadata. */
function inspectionFingerprints(label, value) {
  if (value === null) return node("p", `${label}: no declared fingerprint set.`);
  const section = inspectionMetadata(label, [["Source SHA-256", value?.source_sha256]]);
  if (value && (!inspectionIdentity(value.source_sha256, true) || !Array.isArray(value.generated_artifacts) || value.generated_artifacts.length > 128 || value.generated_artifacts.some(item => typeof item?.path !== "string" || !inspectionIdentity(item.sha256, true)))) throw new Error("The record returned unsupported fingerprint metadata.");
  section.append(table(`${label}: generated artifacts`, [["Declared relative identity", "path"], ["SHA-256", "sha256"]], value?.generated_artifacts || []));
  return section;
}

/** Keep unresolved lifecycle finding tokens separate from any selected impact comparison. */
function inspectionReferences(values) {
  if (!Array.isArray(values) || values.some(value => typeof value?.finding_id !== "string" || value.binding !== "unresolved")) throw new Error("The record returned unsupported impact references.");
  const section = node("section"); section.append(node("h3", "Unresolved impact references"),
    node("p", "Finding identifiers have no comparison pair here. They are not joined to a comparison or treated as approval."));
  if (values.length > 131072) throw new Error("The record returned unsupported impact reference counts.");
  section.append(node("pre", values.map(value => `${inspectionValue(value.finding_id)} · unresolved`).join("\n") || "None declared"));
  return section;
}

/** Validate one page's capture/count metadata; filters use this response's complete counts only. */
function checkedInspectionPage(response, kind, query, context) {
  const page = checkedPage(response, 50);
  inspectionRequired(response, ["resource_version", "snapshot_version", "availability", "page", "counts"]);
  if (!inspectionIdentity(response.snapshot_version, true) || !inspectionIdentity(response.resource_version, true) ||
      !["absent-index", "index-upgrade-required", "empty", "available", "needs-attention"].includes(response.availability) ||
      page.next_cursor && page.next_cursor.length > 256) throw new Error("The inspection returned unsupported capture metadata.");
  if (context?.snapshot_version && response.snapshot_version !== context.snapshot_version) {
    const error = new Error("The capture changed. Reload the selected record or comparison before continuing."); error.details = {code:"version-conflict"}; throw error;
  }
  const ceiling = {records:1000, history:1024, queue:64000, comparisons:1000, changes:100000, findings:100000, prior:100000}[kind];
  const keys = {records:["registered_records", "matching_records", "unavailable_records"], history:["total_events"],
    queue:["distinct_records", "total_owner_placements", "matching_owner_placements", "total_groups", "matching_groups"],
    comparisons:["registered_comparisons", "unavailable_comparisons"], changes:["total_changes", "matching_changes"],
    findings:["total_findings", "matching_findings"], prior:["total_prior_dispositions"]}[kind];
  if (!keys || keys.some(key => !Number.isSafeInteger(response.counts?.[key]) || response.counts[key] < 0 || response.counts[key] > (key === "distinct_records" ? 1000 : ceiling))) throw new Error("The inspection returned unsupported whole-scope counts.");
  const matching = {records:"matching_records", history:"total_events", queue:"matching_owner_placements", comparisons:"registered_comparisons", changes:"matching_changes", findings:"matching_findings", prior:"total_prior_dispositions"}[kind];
  const total = {records:"registered_records", history:"total_events", queue:"total_owner_placements", comparisons:"registered_comparisons", changes:"total_changes", findings:"total_findings", prior:"total_prior_dispositions"}[kind];
  if (response.counts[matching] !== page.total_matching || response.counts[total] < page.total_matching ||
      kind === "records" && response.counts.unavailable_records > response.counts.registered_records ||
      kind === "comparisons" && response.counts.unavailable_comparisons > response.counts.registered_comparisons ||
      kind === "queue" && (response.counts.distinct_records > response.counts.total_owner_placements || response.counts.matching_groups > response.counts.total_groups)) throw new Error("The inspection counts could not be reconciled. Retry the read.");
  if (kind === "history" && (response.record_id !== context.record_id || response.as_of !== null)) throw new Error("The history belongs to a different registered record or date context.");
  if ((kind === "records" || kind === "queue") && response.as_of !== (query.as_of || null)) throw new Error("The inspection returned a different explicit review date.");
  if (kind === "records" && !query.as_of && page.items.some(item => item.derived_status !== null)) throw new Error("An inventory without a date cannot report computed lifecycle status.");
  if (["changes", "findings", "prior"].includes(kind) && (response.comparison_id !== context.comparison_id || response.comparison_version !== context.resource_version)) throw new Error("The inspection returned a foreign comparison context.");
  if (kind === "findings") {
    inspectionSummary(response.full_summary);
    const summaryKeys = ["old_controls", "new_controls", "added", "removed", "content_changed", "identity_migrated", "unchanged", "findings", "blocking", "review_required", "informational", "dispositioned_resolved", "dispositioned_accepted_risk", "dispositioned_still_open", "undispositioned"];
    const dispositionKeys = ["resolved", "accepted_risk", "still_open", "undispositioned"];
    if (summaryKeys.some(key => response.full_summary[key] !== context.summary[key]) || response.full_summary.findings !== response.counts.total_findings ||
        dispositionKeys.some(key => !Number.isSafeInteger(response.emitted_dispositions?.[key]) || response.emitted_dispositions[key] < 0) ||
        dispositionKeys.reduce((sum, key) => sum + response.emitted_dispositions[key], 0) !== response.counts.matching_findings) throw new Error("The filtered finding scope does not reconcile with the complete comparison summary.");
    if (page.items.some(item => item.comparison_id !== context.comparison_id || item.comparison_version !== context.resource_version)) throw new Error("A finding belongs to a different comparison pair or capture.");
  }
  const rowFields = {records:["record_id", "record_version", "resource_id", "policy_key", "version_key", "state", "derived_status", "owner_keys", "next_review_date", "availability", "diagnostic_code", "validation_state"],
    history:["event_id", "sequence", "timestamp", "previous_state", "next_state", "actor_key", "declared_role", "assertions", "impact_references", "replacement"],
    queue:["record_id", "resource_id", "owner_key", "next_review_date", "policy_key", "version_key", "state", "derived_status", "blockers"],
    comparisons:["comparison_id", "resource_id", "manifest_sha256", "old", "new", "availability", "diagnostic_code", "freshness"],
    changes:["subject_id", "change_class", "old_sha256", "new_sha256", "old_subjects", "new_subjects", "migration"],
    findings:["finding_id", "comparison_id", "comparison_version", "priority", "reason_code", "required_action", "subject_id", "change_class", "old_sha256", "new_sha256", "old_subjects", "new_subjects", "migration", "framework_groups", "affected_artifact_id", "dependency_id", "policy_resource_identity", "prior_gap_classification", "prior_decision_state", "owner", "policy_sources", "disposition"],
    prior:["finding_id", "status", "decided_by", "decided_at"]};
  for (const item of page.items) {
    inspectionRequired(item, rowFields[kind]);
    if (["records", "queue"].includes(kind) && (!inspectionIdentity(item.record_id) || !inspectionIdentity(item.resource_id)) ||
        kind === "comparisons" && (!inspectionIdentity(item.comparison_id) || !inspectionIdentity(item.resource_id))) throw new Error("The inspection returned an unsupported registered identity.");
  }
  return page;
}

/** Label every entity-specific count without conflating distinct records, placements, groups or filtered findings. */
function inspectionCounts(response, kind) {
  const labels = {records:[["registered_records", "registered records"], ["matching_records", "matching records"], ["unavailable_records", "unavailable records"]],
    history:[["total_events", "recorded events"]], queue:[["distinct_records", "distinct records"], ["total_owner_placements", "total owner placements"], ["matching_owner_placements", "matching owner placements"], ["total_groups", "total owner groups"], ["matching_groups", "matching owner groups"]],
    comparisons:[["registered_comparisons", "registered comparisons"], ["unavailable_comparisons", "unavailable comparisons"]],
    changes:[["total_changes", "complete changes"], ["matching_changes", "matching changes"]], findings:[["total_findings", "complete findings"], ["matching_findings", "matching findings"]], prior:[["total_prior_dispositions", "prior-only dispositions"]]};
  return labels[kind].map(([key, label]) => `${response.counts[key]} ${label}`).join(" · ");
}

/** Render bounded selected rows from explicit metadata allowlists; all strings use textContent. */
function inspectionRows(kind, rows, select) {
  const display = node("div");
  if (!rows.length) {display.append(node("p", "No matching metadata rows.", "empty")); return display;}
  if (kind === "queue") {
    const groups = new Map();
    for (const row of rows) {if (!groups.has(row.owner_key)) groups.set(row.owner_key, []); groups.get(row.owner_key).push(row);}
    for (const [owner, items] of groups) {
      display.append(node("h3", `Declared owner ${inspectionValue(owner)} — this page's placements`),
        table("Lifecycle owner placements on this page", [["Record", "record_id"], ["Policy", "policy_key"], ["Version", "version_key"], ["Stored state", "state"], ["Computed status", "derived_status"], ["Next review", "next_review_date"], ["Blockers", "blockers"]], items.map(item => ({...item, blockers:inspectionTokens(item.blockers)}))));
      for (const item of items) display.append(button(`Inspect record ${item.record_id}`, () => select(item.record_id)));
    }
    return display;
  }
  for (const row of rows) {
    const article = node("article");
    if (kind === "records") {
      article.append(inspectionMetadata(`Recorded lifecycle ${row.record_id}`, [["Registered resource", row.resource_id], ["Record version", row.record_version], ["Policy key", row.policy_key], ["Version key", row.version_key], ["Stored state", row.state], ["Computed status", row.derived_status === null ? "Not computed without an explicit date" : row.derived_status], ["Declared owners", inspectionTokens(row.owner_keys)], ["Next review", row.next_review_date], ["Availability", row.availability], ["Admission", row.validation_state], ["Diagnostic code", row.diagnostic_code]]), button(`Inspect record ${row.record_id}`, () => select(row.record_id)));
    } else if (kind === "comparisons") {
      article.append(inspectionMetadata(`Declared comparison ${row.comparison_id}`, [["Registered manifest", row.resource_id], ["Manifest SHA-256", row.manifest_sha256], ["Availability", row.availability], ["Diagnostic code", row.diagnostic_code], ["Freshness", "Not computed from inventory"]]), inspectionPair("Declared old fingerprint", row.old), inspectionPair("Declared new fingerprint", row.new), button(`Inspect comparison ${row.comparison_id}`, () => select(row.comparison_id)));
    } else if (kind === "history") {
      article.append(inspectionMetadata(`Recorded transition ${row.event_id}`, [["Sequence", row.sequence], ["Timestamp", row.timestamp], ["Previous state", row.previous_state], ["Next state", row.next_state], ["Declared actor", row.actor_key], ["Declared role", row.declared_role], ["Replacement policy", row.replacement?.policy_key], ["Replacement version", row.replacement?.version_key]]),
        table("Declared transition assertions", [["Actor key", "actor_key"], ["Declared role", "declared_role"]], row.assertions || []), inspectionReferences(row.impact_references || []));
    } else if (kind === "changes" || kind === "findings") {
      article.append(inspectionMetadata(kind === "findings" ? `Finding ${row.finding_id}` : `Change ${row.subject_id}`, [...(kind === "findings" ? [["Comparison", row.comparison_id], ["Comparison version", row.comparison_version]] : []), ["Subject", row.subject_id], ["Change class", row.change_class], ["Old SHA-256", row.old_sha256], ["New SHA-256", row.new_sha256]]),
        inspectionSubjects("Old subject fingerprints", row.old_subjects), inspectionSubjects("New subject fingerprints", row.new_subjects));
      if (row.migration) article.append(inspectionMetadata("Declared migration metadata", [["Relationship", row.migration.relationship], ["Declared approver", row.migration.approved_by], ["Recorded approval time", row.migration.approved_at]]));
      if (kind === "findings") article.append(inspectionMetadata("Finding review metadata", [["Priority", row.priority], ["Reason code", row.reason_code], ["Required action", row.required_action], ["Framework groups", inspectionTokens(row.framework_groups)], ["Affected artifact", row.affected_artifact_id], ["Dependency", row.dependency_id], ["Policy resource identity", row.policy_resource_identity], ["Prior gap classification", row.prior_gap_classification], ["Prior decision state", row.prior_decision_state], ["Declared owner", row.owner], ["Policy source identities", inspectionTokens(row.policy_sources)], ["Disposition", row.disposition?.status], ["Declared decision actor", row.disposition?.decided_by], ["Recorded decision time", row.disposition?.decided_at]]));
    } else if (kind === "prior") article.append(inspectionMetadata(`Prior-only disposition ${row.finding_id}`, [["Status", row.status], ["Declared decision actor", row.decided_by], ["Recorded decision time", row.decided_at]]));
    display.append(article);
  }
  return display;
}

/** Retire only in-flight S3 child reads when their parent starts a successor capture. */
function retireInspectionChildren(container) {
  for (const section of container.querySelectorAll("[data-inspection-page]")) inspectionReaders.get(section)?.();
}

/** Own one bounded S3 page chain without changing the legacy pager or its denominator rules. */
function inspectionPage(path, caption, kind, initialQuery = {}, options = {}) {
  let owner = pending; let generation = 0; let busy = false; let index = 0; let cursors = [null];
  let nextCursor = null; let chainVersion = null; let chainSnapshot = null; let summary = "";
  let applied = {...initialQuery}; let lastRequest = {index:0, cursors:[null], query:{...initialQuery}, fresh:true};
  const section = node("section"); section.setAttribute("data-paged-table", ""); section.setAttribute("data-inspection-page", kind);
  const heading = node("h3", caption); heading.tabIndex = -1;
  const form = node("form"); form.addEventListener("submit", event => event.preventDefault());
  const status = node("p"); status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite"); status.setAttribute("aria-atomic", "true"); status.setAttribute("data-page-status", "");
  const error = node("div"); error.hidden = true; error.tabIndex = -1; error.setAttribute("role", "alert"); error.setAttribute("data-page-error", "");
  const display = node("div"); display.tabIndex = -1; display.setAttribute("role", "region"); display.setAttribute("aria-label", caption); display.setAttribute("data-page-results", "");
  const actions = []; const pane = {section, response:null, failure:null, ready:Promise.resolve(false), read:load};
  /** Consume handled read results; preserve false only for an explicit dirty-edit cancellation. */
  function pageAction(label, action, cancelAware = false) {
    const control = button(label, async () => {
      if (!busy && !stopped && owner === pending && options.current?.() !== false && section.isConnected) {
        const outcome = await action(); if (cancelAware && outcome === false) return false;
      }
    });
    actions.push(control); return control;
  }
  const choices = (options.filters || []).map(([key, label, values]) => {
    const input = fieldInput(form, label, values ? "select" : "text", values ? [["", "All"], ...values.map(value => [value, value])] : undefined);
    input.required = false; input.value = initialQuery[key] || ""; return [key, input];
  });
  const previous = pageAction(`Previous ${caption} page`, () => load(index - 1, cursors, applied, true)); previous.hidden = true;
  const next = pageAction(`Next ${caption} page`, () => {const target = cursors.slice(); target[index + 1] = nextCursor; return load(index + 1, target, applied, true);}); next.hidden = true;
  const retry = pageAction(`Retry ${caption}`, () => load(lastRequest.index, lastRequest.cursors, lastRequest.query, true, lastRequest.fresh)); retry.hidden = true;
  const restart = pageAction(`Restart ${caption} from first page`, () => options.reload ? options.reload() : load(0, [null], lastRequest.query, true, true)); restart.hidden = true;
  if (choices.length) form.append(pageAction(`Apply ${caption} filters`, async () => {
    if (!await allowViewChange()) return false;
    const query = {...initialQuery}; for (const [key, input] of choices) {if (input.value) query[key] = input.value; else delete query[key];}
    await load(0, [null], query, true, true);
  }, true));
  /** Keep previous verified rows on error; obsolete reads cannot reset newer busy state or focus. */
  async function load(targetIndex, targetCursors, query, focusResults = false, fresh = false) {
    if (stopped || owner !== pending || options.current?.() === false) return false;
    const sequence = ++generation; const attached = section.isConnected; const invocationOwner = owner; const invoker = document.activeElement;
    /** Check local sequence, installed-view epoch and parent selection before every publication. */
    const current = () => !stopped && owner === pending && invocationOwner === pending && sequence === generation && options.current?.() !== false && (!attached || section.isConnected);
    lastRequest = {index:targetIndex, cursors:targetCursors.slice(), query:{...query}, fresh}; pane.failure = null;
    busy = true; actions.forEach(control => control.setAttribute("aria-disabled", "true")); display.setAttribute("aria-busy", "true"); error.hidden = true;
    status.textContent = `${caption}: Reading captured metadata…${summary ? ` Previous capture: ${summary}` : ""}`;
    try {
      const params = new URLSearchParams({page_size:"50"}); for (const [key, value] of Object.entries(query)) if (value) params.set(key, value);
      if (targetCursors[targetIndex]) params.set("cursor", targetCursors[targetIndex]);
      const response = await api(`${path}?${params}`); const page = checkedInspectionPage(response, kind, query, options.context);
      if (!fresh && chainVersion !== null && (response.resource_version !== chainVersion || response.snapshot_version !== chainSnapshot)) {
        const failure = new Error("The captured page chain changed. Restart this read from its first page."); failure.details = {code:"version-conflict"}; throw failure;
      }
      const rendered = inspectionRows(kind, page.items, options.select);
      if (kind === "findings") rendered.prepend(inspectionSummary(response.full_summary), table("Filtered emitted disposition scope", [["Status", "status"], ["Matching findings", "count"]], ["resolved", "accepted_risk", "still_open", "undispositioned"].map(status => ({status, count:response.emitted_dispositions[status]}))));
      if (!current()) return false;
      if (fresh) {chainVersion = response.resource_version; chainSnapshot = response.snapshot_version;}
      index = targetIndex; cursors = targetCursors.slice(); applied = {...query}; nextCursor = page.next_cursor;
      const ownedFocus = document.activeElement === invoker;
      pane.response = response; display.replaceChildren(rendered); summary = `${inspectionCounts(response, kind)} · Page ${index + 1} · Capture ${response.snapshot_version}.`;
      if (kind === "history") summary += " Recorded events; no current status is inferred.";
      if (kind === "findings") summary += " All five filters combine with AND. Complete summary and gates remain unfiltered.";
      status.textContent = `${response.availability}: ${summary}`; retry.hidden = true; restart.hidden = true; previous.hidden = index === 0; next.hidden = !nextCursor;
      if (focusResults && ownedFocus && section.isConnected) display.focus(); return true;
    } catch (failure) {
      if (!current()) return false;
      pane.failure = failure; error.textContent = `${failure instanceof Error ? failure.message : "This captured metadata could not be read."}${summary ? " Previous capture retained; it does not reflect this failed read." : " No metadata was installed."}`;
      error.hidden = false; status.textContent = `${caption}: Read failed.${summary ? ` Previous capture: ${summary}` : ""}`;
      retry.hidden = false; restart.hidden = failure.details?.code !== "version-conflict";
      if (section.isConnected && document.activeElement === invoker) error.focus(); return false;
    } finally {
      if (!stopped && sequence === generation) {busy = false; display.setAttribute("aria-busy", "false"); actions.forEach(control => control.setAttribute("aria-disabled", "false"));}
    }
  }
  /** Make an older child read obsolete even when its parent's view epoch stays unchanged. */
  function retireInspectionPage() {
    generation++; busy = false; display.setAttribute("aria-busy", "false"); actions.forEach(control => control.setAttribute("aria-disabled", "false"));
    if (summary) status.textContent = `Previous capture retained while its parent reloads: ${summary}`;
  }
  inspectionReaders.set(section, retireInspectionPage);
  /** Recover a retained installed pane after a cancelled guarded refresh, without resurrecting an old read. */
  function resumeInspectionPage() {
    if (stopped || !section.isConnected || owner === pending || element("view").inert) return;
    owner = pending; generation++; busy = false; display.setAttribute("aria-busy", "false"); actions.forEach(control => control.setAttribute("aria-disabled", "false"));
    if (summary) status.textContent = `Previous captured metadata retained after the refresh was cancelled: ${summary}`;
  }
  retainedPagers.set(section, resumeInspectionPage);
  section.append(heading, form, status, error, display, previous, next, retry, restart);
  pane.ready = load(0, [null], applied, false, true); return pane;
}

/** Stage a selected record/comparison's whole pane group before replacing an earlier captured view. */
function inspectionSelection(caption, build, isCurrent) {
  let owner = pending; let generation = 0; let busy = false; let previousSummary = ""; let last; let installedGroup;
  const section = node("section"); section.setAttribute("data-paged-table", ""); section.setAttribute("data-inspection-selection", caption);
  const heading = node("h3", caption); heading.tabIndex = -1;
  const status = node("p", "Choose a registered item above."); status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite"); status.setAttribute("aria-atomic", "true");
  const error = node("div"); error.hidden = true; error.tabIndex = -1; error.setAttribute("role", "alert"); error.setAttribute("data-page-error", "");
  const display = node("div"); display.tabIndex = -1; display.setAttribute("role", "region"); display.setAttribute("aria-label", caption);
  const retry = button(`Retry ${caption}`, async () => {if (last && !busy && current()) await read(last);}); retry.hidden = true;
  /** Gate this installed selection independently from other lifecycle/impact lanes. */
  function current() {return !stopped && owner === pending && section.isConnected && isCurrent();}
  /** Invalidate older selection work; errors and retries remain local to this connected region. */
  async function read(context) {
    if (!current()) return false;
    const invocationOwner = owner; const sequence = ++generation; const invoker = document.activeElement; retireInspectionChildren(display); last = {...context}; busy = true;
    display.setAttribute("aria-busy", "true"); retry.setAttribute("aria-disabled", "true"); error.hidden = true;
    status.textContent = `Reading ${caption}…${previousSummary ? ` Previous capture: ${previousSummary}` : ""}`;
    /** Recheck the selection identity after every staged asynchronous read. */
    const owned = () => current() && invocationOwner === pending && sequence === generation;
    try {
      const group = {};
      /** Retained verified subpages remain usable after a cancelled refresh; only the current staged build can install a successor. */
      const paneCurrent = () => owned() || current() && !busy && installedGroup === group;
      const staged = await build(context, owned, () => read({...context}), paneCurrent);
      if (!owned()) return false;
      const ownedFocus = document.activeElement === invoker;
      installedGroup = group; display.replaceChildren(staged); previousSummary = `${context.id}${context.as_of ? ` · as of ${context.as_of}` : context.kind === "comparison" ? " · selected old/new comparison pair" : " · recorded history only"}`;
      status.textContent = `${caption}: ${previousSummary}. Captured metadata does not authenticate approval.`; retry.hidden = true; if (ownedFocus) heading.focus(); return true;
    } catch (failure) {
      if (!owned()) return false;
      error.textContent = `${failure instanceof Error ? failure.message : "The selected metadata could not be read."}${previousSummary ? " Previous capture retained; it does not reflect this failed read." : " No selected metadata was installed."}`;
      error.hidden = false; retry.hidden = false; status.textContent = `${caption}: Read failed.`; if (document.activeElement === invoker) error.focus(); return false;
    } finally {
      if (!stopped && sequence === generation) {busy = false; display.setAttribute("aria-busy", "false"); retry.setAttribute("aria-disabled", "false");}
    }
  }
  /** Rebind a retained selection and invalidate its former pending build without moving focus. */
  function resumeInspectionSelection() {
    if (stopped || !section.isConnected || owner === pending || element("view").inert) return;
    owner = pending; generation++; busy = false; display.setAttribute("aria-busy", "false"); retry.setAttribute("aria-disabled", "false");
    if (previousSummary) status.textContent = `Previous capture retained after cancelled refresh: ${previousSummary}.`;
  }
  retainedPagers.set(section, resumeInspectionSelection); section.append(heading, status, error, display, retry);
  return {section, read};
}

/** Read lifecycle inventory/history/status/owner queues and impact comparisons without effects or implicit dates. */
async function lifecycleImpactView() {
  if (!inspectionSupported()) return node("p", "Lifecycle & Impact requires negotiated API2 2.1.0, 2.2.0 or 2.3.0. Relaunch with matching assets.", "empty");
  let owner = pending; let lifecycleGeneration = 0; let selectedRecord = null; let appliedDate = null; let installedLifecycleGroup; let lifecycleStaging = false;
  const root = node("section"); root.setAttribute("data-inspection-panel", ""); root.setAttribute("data-paged-table", "");
  root.append(node("p", "Read-only captured lifecycle and framework-impact metadata. Keys, identities and hashes can be sensitive. Declared actors, owners, states and dispositions are unauthenticated; these views grant no transition, approval, export or write authority.", "muted"));
  /** Keep the installed view owner separate from per-pane request sequences. */
  const current = () => !stopped && owner === pending && root.isConnected;
  /** Recover retained controls after a cancelled guarded view refresh without making former work current. */
  function resumeInspectionView() {if (!stopped && root.isConnected && owner !== pending && !element("view").inert) owner = pending;}
  retainedPagers.set(root, resumeInspectionView);
  const lifecycle = node("section"); lifecycle.append(node("h2", "Lifecycle records and owner queue"));
  const form = node("form"); form.addEventListener("submit", event => event.preventDefault());
  const date = fieldInput(form, "Explicit lifecycle review date", "date"); date.required = false;
  const lifecycleOwner = fieldInput(form, "Lifecycle owner key"); lifecycleOwner.required = false;
  const lifecycleState = fieldInput(form, "Stored lifecycle state", "select", [["", "All"], ...["draft", "in-review", "approved", "superseded", "retired"].map(state => [state, state])]); lifecycleState.required = false;
  const dateError = node("div"); dateError.hidden = true; dateError.tabIndex = -1; dateError.setAttribute("role", "alert");
  const inventories = node("div"); let initialRecords;
  /** Read the chosen record/history at one date/capture; unresolved finding IDs never select an impact pair. */
  async function buildRecord(context, owned, reload, paneCurrent) {
    const stage = node("div"); let detail;
    if (context.as_of) {
      detail = await api(`/lifecycle/records/${encodeURIComponent(context.id)}?${new URLSearchParams({as_of:context.as_of})}`);
      inspectionRequired(detail, ["resource_version", "snapshot_version", "record_id", "resource_id", "policy_key", "version_key", "as_of", "state", "derived_status", "owner_keys", "next_review_date", "blockers", "current_fingerprints", "approved_fingerprints", "artifact_identity_changes", "replaced_by", "replacement_record_id", "impact_references", "provenance", "trust_boundary"]);
      if (!owned()) throw new Error("The selected record read was superseded.");
      if (detail.record_id !== context.id || detail.as_of !== context.as_of || !inspectionIdentity(detail.snapshot_version, true) || !inspectionIdentity(detail.resource_version, true)) throw new Error("The record returned a foreign capture or date.");
      stage.append(inspectionMetadata("Captured lifecycle status", [["Record", detail.record_id], ["Registered resource", detail.resource_id], ["Policy key", detail.policy_key], ["Version key", detail.version_key], ["As of", detail.as_of], ["Stored state", detail.state], ["Computed status", detail.derived_status], ["Declared owners", inspectionTokens(detail.owner_keys)], ["Next review", detail.next_review_date], ["Blockers", inspectionTokens(detail.blockers)], ["Identity changes", inspectionTokens(detail.artifact_identity_changes)], ["Replacement policy", detail.replaced_by?.policy_key], ["Replacement version", detail.replaced_by?.version_key], ["Replacement registered record", detail.replacement_record_id]]),
        inspectionFingerprints("Current captured fingerprints", detail.current_fingerprints), inspectionFingerprints("Declared approved fingerprints", detail.approved_fingerprints), inspectionReferences(detail.impact_references), inspectionProvenance(detail.provenance));
    } else stage.append(node("p", "Choose an explicit review date before requesting computed lifecycle status. This selection shows recorded transition history only."));
    const history = inspectionPage(`/lifecycle/records/${encodeURIComponent(context.id)}/history`, "Lifecycle transition history", "history", {}, {context:detail || {record_id:context.id}, current:paneCurrent, reload});
    if (!await history.ready) throw history.failure || new Error("The transition history could not be staged at this record's capture. Retry the selected record.");
    stage.append(history.section); return stage;
  }
  const record = inspectionSelection("Selected lifecycle record", buildRecord, current);
  /** Retain the exact date and local read-error focus; false is reserved for Keep editing. */
  async function selectRecord(id) {
    if (!current() || lifecycleStaging) return;
    if (!await allowViewChange()) return false;
    selectedRecord = id; await record.read({id, as_of:appliedDate});
  }
  /** Replace inventory and queue together under a new explicit date/filter context. */
  async function installLifecycle(dateValue, ownerValue, stateValue, focus = false) {
    const generation = ++lifecycleGeneration; const invocationOwner = owner; const invoker = document.activeElement; const group = {};
    retireInspectionChildren(inventories); lifecycleStaging = true;
    try {
    /** Bind new inventory/queue controls to this date/filter context and current installed view epoch. */
    const owned = () => !stopped && owner === pending && (generation === lifecycleGeneration || !lifecycleStaging && group === installedLifecycleGroup);
    /** Only this initiating view epoch may install the staged inventory group. */
    const mayInstall = () => owned() && invocationOwner === pending && generation === lifecycleGeneration;
    const query = {}; if (dateValue) query.as_of = dateValue; if (ownerValue) query.owner = ownerValue; if (stateValue) query.state = stateValue;
    const records = inspectionPage("/lifecycle/records", "Lifecycle record inventory", "records", query, {current:owned, select:selectRecord});
    const stage = node("div"); stage.append(records.section);
    let queue;
    if (dateValue) {const queueQuery = {as_of:dateValue}; if (ownerValue) queueQuery.owner = ownerValue;
      queue = inspectionPage("/lifecycle/queue", "Lifecycle owner queue", "queue", queueQuery, {current:owned, select:selectRecord}); stage.append(queue.section);
    } else stage.append(node("p", "Owner queue and computed status require an explicit review date. No clock date is selected automatically."));
    const results = await Promise.all([records.ready, ...(queue ? [queue.ready] : [])]);
    const sameCapture = !queue || records.response?.snapshot_version === queue.response?.snapshot_version;
    if (results.some(value => !value) || !sameCapture) {
      if (mayInstall()) {
        if (!inventories.children.length) {installedLifecycleGroup = group; inventories.replaceChildren(stage);}
        else {dateError.textContent = "The lifecycle inventory and owner queue could not be staged at one capture. Previous date/filter results are retained. Retry Apply lifecycle date and filters."; dateError.hidden = false; if (root.isConnected && document.activeElement === invoker) dateError.focus();}
        if (focus && root.isConnected && !dateError.hidden && document.activeElement === invoker) dateError.focus();
      }
      return false;
    }
    if (!mayInstall()) return false;
    installedLifecycleGroup = group; appliedDate = dateValue; inventories.replaceChildren(stage);
    if (selectedRecord && current()) await record.read({id:selectedRecord, as_of:appliedDate});
    if (focus && current() && document.activeElement === invoker) records.section.querySelector("[data-page-results]").focus(); return true;
    } finally {if (generation === lifecycleGeneration) lifecycleStaging = false;}
  }
  const apply = button("Apply lifecycle date and filters", async () => {
    if (!current()) return;
    if (!await allowViewChange()) return false;
    dateError.hidden = true;
    if (date.value && (!/^[0-9]{4}-[0-9]{2}-[0-9]{2}$/.test(date.value) || !date.reportValidity())) {dateError.textContent = "Use an explicit valid YYYY-MM-DD review date."; dateError.hidden = false; dateError.focus(); return;}
    await installLifecycle(date.value || null, lifecycleOwner.value, lifecycleState.value, true);
  }); form.append(apply); lifecycle.append(form, dateError, inventories, record.section);
  const impact = node("section"); impact.append(node("h2", "Framework impact comparisons"), node("p", "Inventory old/new fingerprints are declared, not computed. Select one exact comparison to read captured changes/findings and prior-only dispositions. Unresolved lifecycle finding IDs are not joined here.", "muted"));
  /** Stage detail and all three bounded subpages from the same capture and exact old/new comparison pair. */
  async function buildComparison(context, owned, reload, paneCurrent) {
    const detail = await api(`/framework-impact/comparisons/${encodeURIComponent(context.id)}`);
    inspectionRequired(detail, ["resource_version", "snapshot_version", "comparison_id", "resource_id", "freshness", "old", "new", "summary", "provenance", "prior_report_sha256", "prior_report_admission", "trust_boundary"]);
    if (!owned()) throw new Error("The comparison read was superseded.");
    if (detail.comparison_id !== context.id || !inspectionIdentity(detail.snapshot_version, true) || !inspectionIdentity(detail.resource_version, true) || detail.freshness !== "captured-current") throw new Error("The comparison returned a foreign capture or freshness state.");
    const stage = node("div"); stage.append(inspectionMetadata("Captured comparison context", [["Comparison", detail.comparison_id], ["Registered manifest", detail.resource_id], ["Comparison version", detail.resource_version], ["Snapshot", detail.snapshot_version], ["Freshness", detail.freshness], ["Prior report SHA-256", detail.prior_report_sha256], ["Prior report admission", detail.prior_report_admission]]), inspectionPair("Captured old resource", detail.old), inspectionPair("Captured new resource", detail.new), inspectionSummary(detail.summary), inspectionProvenance(detail.provenance));
    const options = {context:detail, current:paneCurrent, reload};
    const changes = inspectionPage(`/framework-impact/comparisons/${encodeURIComponent(context.id)}/changes`, "Framework impact changes", "changes", {}, {...options, filters:[["change_class", "Impact change class", ["added", "removed", "content-changed", "identity-migrated", "unchanged"]]]});
    const findings = inspectionPage(`/framework-impact/comparisons/${encodeURIComponent(context.id)}/findings`, "Framework impact findings", "findings", {}, {...options, filters:[["group", "Impact framework group"], ["decision_state", "Impact prior decision state", ["applicable", "not-applicable", "deferred", "under-review"]], ["policy_source", "Impact policy source identity"], ["priority", "Impact finding priority", ["blocking", "review-required", "informational"]], ["owner", "Impact owner key"]]});
    const prior = inspectionPage(`/framework-impact/comparisons/${encodeURIComponent(context.id)}/prior-dispositions`, "Prior-only impact dispositions", "prior", {}, options);
    const results = await Promise.all([changes.ready, findings.ready, prior.ready]);
    if (results.some(value => !value)) throw [changes, findings, prior].find((pane, index) => !results[index])?.failure || new Error("The comparison's changes, findings and prior history could not be staged at one capture. Retry the selected comparison.");
    stage.append(changes.section, findings.section, prior.section); return stage;
  }
  const comparison = inspectionSelection("Selected framework impact comparison", buildComparison, current);
  /** Preserve local comparison failure focus; only explicit dirty cancellation restores the trigger. */
  async function selectComparison(id) {
    if (!current()) return;
    if (!await allowViewChange()) return false;
    await comparison.read({id, kind:"comparison"});
  }
  const comparisons = inspectionPage("/framework-impact/comparisons", "Framework comparison inventory", "comparisons", {}, {select:selectComparison});
  impact.append(comparisons.section, comparison.section); root.append(lifecycle, impact);
  initialRecords = installLifecycle(null, "", ""); await Promise.all([initialRecords, comparisons.ready]);
  return root;
}

/** Announce the locked state and place initial keyboard focus on the credential field. */
function initializeUnlock() {
  element("status").textContent = "Workspace locked — passphrase required.";
  element("passphrase").setAttribute("aria-describedby", "status");
  element("passphrase").focus();
}

/** Guard one pending unlock, retain trigger focus, and announce server throttle facts verbatim. */
async function submitUnlock(event) {
  event.preventDefault();
  if (unlockPending || stopped) return;
  const form = event.currentTarget;
  const submit = form.querySelector("button");
  const field = element("passphrase");
  const invoker = document.activeElement;
  unlockPending = true; form.setAttribute("aria-busy", "true"); submit.setAttribute("aria-disabled", "true");
  element("error").hidden = true;
  element("status").textContent = "Unlocking workspace…";
  try {
    if ([...field.value].length < 15 || [...field.value].length > 128) throw new Error("Use 15–128 characters.");
    const response = await api("/session/unlock", "POST", {passphrase: field.value});
    field.value = "";
    if (stopped) return;
    if (response.session.api_major !== apiMajor || apiMajor === 2 && response.session.contract_version !== apiContractVersion) throw new Error(apiMajor === 1 ? "This UI requires API version 1. Install matching workspace assets." : "This UI requires the selected API version 2. Install matching workspace assets.");
    capability = response.capability;
    readOnly = response.session.read_only;
    element("unlock-panel").hidden = true;
    element("workspace").hidden = false;
    element("stop").hidden = false;
    element("connection").textContent = response.session.read_only ? "Read-only · Local" : "Local session";
    if (await renderView()) element("main").focus();
  } catch (error) {
    field.value = "";
    if (stopped) return;
    if (error.details?.code === "unlock-throttled" && typeof error.details.message === "string") {
      element("status").textContent = error.details.message;
      if ((invoker === field || invoker === submit) && document.activeElement === invoker && field.isConnected && !element("unlock-panel").hidden) field.focus();
    } else { element("status").textContent = "Workspace locked — passphrase required."; showError(error); }
  } finally { unlockPending = false; form.setAttribute("aria-busy", "false"); submit.setAttribute("aria-disabled", "false"); }
}
element("unlock-form").addEventListener("submit", submitUnlock);
initializeUnlock();

for (const title of titles) {
  const button = node("button", title);
  if (title === activeView) button.setAttribute("aria-current", "page");
  button.addEventListener("click", () => navigate(title));
  element("navigation").append(button);
}
element("refresh").addEventListener("click", () => navigate(activeView,viewFilters));
element("stop").addEventListener("click", () => element("stop-dialog").showModal());
element("keep-working").addEventListener("click", () => element("stop-dialog").close());
element("confirm-stop").addEventListener("click", async () => {
  try { await api("/session/shutdown", "POST", {}); stopped = true; invalidateBundlePanels(); stopSourceRestoreRows(); pending++; capability = "";
    for (const row of operationRows.values()) { row.generation++; row.status.textContent = "Workspace stopped. Operation status is no longer queryable in this session."; row.cancel.setAttribute("aria-disabled", "true"); row.check.setAttribute("aria-disabled", "true"); row.review.disabled = true; }
    for (const row of requestRows.values()) { row.status.textContent = "Workspace stopped. The request cannot be recovered in this session."; row.retry.setAttribute("aria-disabled", "true"); row.review.setAttribute("aria-disabled", "true"); if (row.path === "/project/source-bundle-imports") { row.prepared = null; row.reviewContext = null; } }
    element("stop-dialog").close(); element("workspace").hidden = true; element("stop").hidden = true; element("connection").textContent = "Stopped"; element("status").textContent = "Workspace stopped. Relaunch it from the terminal to continue."; element("main").focus(); }
  catch (error) { element("stop-dialog").close(); showError(error); }
});


/** Require an explicit discard decision before replacing a form with unsaved edits. */
async function allowViewChange() {
  return !dirty || await confirmDiscard();
}

/** Put successful destination focus on its heading; failures keep their error focus. */
function focusViewTitle() {
  const heading = element("view-title");
  heading.tabIndex = -1;
  heading.focus();
}

/** Return false only for cancellation; count activation may request result-list focus. */
async function navigate(title, filters = {}, focusResults = false) {
  if (!await allowViewChange()) return false;
  dirty = false; activeView = title; viewFilters = filters;
  for (const sibling of element("navigation").children) {
    if (sibling.textContent === title) sibling.setAttribute("aria-current", "page"); else sibling.removeAttribute("aria-current");
  }
  if (await renderView()) {
    const results = focusResults && element("view").querySelector("[data-page-results]");
    if (results) results.focus(); else focusViewTitle();
  }
}

/** Native Escape/close keeps edits; only Discard edits permits a view replacement. */
function confirmDiscard() {
  const invoker = document.activeElement;
  const dialog=node("dialog");dialog.setAttribute("aria-labelledby","discard-title");
  const heading=node("h2","Discard unconfirmed edits?");heading.id="discard-title";heading.tabIndex=-1;
  dialog.append(heading,node("p","Only this page's unconfirmed form changes will be discarded. Saved files remain unchanged."));
  return new Promise(resolve=>{
    let confirmed=false;dialog.append(button("Keep editing",()=>dialog.close()),button("Discard edits",()=>{confirmed=true;dialog.close();}));
    dialog.addEventListener("close",()=>{
      dialog.remove();
      if(!confirmed && invoker?.isConnected && !invoker.disabled)invoker.focus();
      resolve(confirmed);
    },{once:true});document.body.append(dialog);dialog.showModal();heading.focus();
  });
}

/** Keep the invoking control focused while busy; guard duplicate activation and restore cancelled navigation. */
function button(label, action) {
  const control = node("button", label); control.type = "button";
  control.setAttribute("aria-disabled", "false");
  control.addEventListener("click", async () => {
    if (control.getAttribute("aria-disabled") === "true") return;
    control.setAttribute("aria-disabled", "true"); control.setAttribute("aria-busy", "true");
    let cancelled = false;
    try {cancelled = await action() === false;} catch(error) {showError(error);}
    finally {control.setAttribute("aria-disabled", "false");control.setAttribute("aria-busy", "false");if(cancelled && control.isConnected)control.focus();}
  });
  return control;
}
function fieldInput(form, label, type = "text", options) {
  const id = `field-${crypto.randomUUID()}`;
  const caption = node("label", label); caption.htmlFor = id;
  const input = node(type === "select" ? "select" : type === "textarea" ? "textarea" : "input");
  input.id = id; input.required = true;
  if (type !== "select" && type !== "textarea") input.type = type;
  if (type === "textarea") {input.rows = 18; input.spellcheck = false;}
  if (options) for (const [value,label] of options) {const option = node("option",label);option.value = value;input.append(option);}
  form.append(caption,input); return input;
}
function field(form, label, type = "text", options) {
  const input = fieldInput(form, label, type, options);
  input.addEventListener("input", () => {dirty = true;});
  return input;
}
/** Reuse one session-owned region outside the replaceable view and global status. */
function operationRegion() {
  let region = element("operation-region");
  if (!region) { region = node("section"); region.id = "operation-region"; region.setAttribute("aria-label", "Workspace operations"); element("main").append(region); }
  return region;
}

/** Keep bounded recovery for an unacknowledged request using its original replay key. */
function pendingRequestRow(key, path, retry) {
  if (requestRows.size >= 256) throw new Error("The supported request display bound was reached. Resolve pending requests or relaunch the workspace.");
  const article = node("article"); article.hidden = true; article.setAttribute("data-pending-request-id", key);
  const status = node("p"); status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite"); status.setAttribute("aria-atomic", "true"); status.setAttribute("data-request-status", ""); status.tabIndex = -1; status.setAttribute("aria-label", `Preparation request ${path} status`);
  const error = node("div"); error.hidden = true; error.setAttribute("role", "alert"); error.setAttribute("data-request-error", "");
  const row = { key, path, article, status, error, busy: false, prepared: null, reviewContext: null };
  /** Prevent duplicate activation while keeping the initiating native control focused. */
  const requestAction = (label, action) => {
    const control = node("button", label); control.type = "button"; control.setAttribute("aria-disabled", "false");
    control.addEventListener("click", async () => {
      if (stopped || row.busy || requestRows.get(key) !== row || control.getAttribute("aria-disabled") === "true") return;
      row.busy = true; row.retry.setAttribute("aria-disabled", "true"); row.review.setAttribute("aria-disabled", "true");
      try { await action(); } catch (failure) { if (!stopped && requestRows.get(key) === row) pendingRequestFailure(row, failure); }
      finally { row.busy = false; if (!stopped && requestRows.get(key) === row) { row.retry.setAttribute("aria-disabled", "false"); row.review.setAttribute("aria-disabled", "false"); } }
    }); return control;
  };
  row.retry = requestAction("Retry the same request", retry);
  row.review = requestAction("Review prepared write", async () => {
    if (!row.prepared || !await allowViewChange()) return;
    const origin = pending;
    const opened = await preview(row.prepared, undefined, () => !stopped && requestRows.get(key) === row && origin === pending && !document.querySelector("dialog[open]"), row.reviewContext);
    if (opened && !stopped && requestRows.get(key) === row) { row.error.hidden = true; row.status.textContent = "Preparation is ready for review; no write has been confirmed."; }
  }); row.review.hidden = true;
  article.append(node("h2", `Preparation request: ${path}`), status, error, row.retry, row.review); operationRegion().append(article); requestRows.set(key, row); return row;
}

/** Announce owned recovery and typed bundle rejections without changing a newer view or focus. */
function pendingRequestFailure(row, failure) {
  row.article.hidden = false; row.error.hidden = false; row.error.textContent = failure instanceof Error ? failure.message : "The preparation could not be read.";
  row.status.textContent = row.prepared ? "Preparation is ready. The prepared write could not be loaded; review it again before confirming." :
    ["/project/bundle-imports","/project/bundle-exports","/project/source-bundle-imports","/project/source-bundle-exports"].includes(row.path) && failure.details?.code === "bundle-preparation-in-progress" ? "Preparation remains in progress. Retry the same immutable request before preparing another write." :
    ["/project/bundle-imports","/project/bundle-exports","/project/source-bundle-imports","/project/source-bundle-exports"].includes(row.path) && typeof failure.details?.code === "string" && failure.details?.retryable === false ? "The server rejected this request. Review its inputs and prepare a new request; no write has been confirmed." :
    failure.details?.retryable === false ? "The request response is unsupported. Relaunch the workspace before preparing another write." :
    "The request outcome is unknown. No operation ID was received. Retry the same request before preparing another write.";
  row.retry.hidden = !!row.prepared || failure.details?.retryable === false; row.review.hidden = !row.prepared;
}

/** Release a request slot and transfer only the focus that belonged to that removed row. */
function removePendingRequest(row, destination) {
  const transferFocus = !stopped && row.article.contains(document.activeElement) && destination?.isConnected;
  requestRows.delete(row.key); row.article.remove();
  if (transferFocus) destination.focus();
}

/** Create bounded, session-owned operation rows that navigation cannot overwrite. */
function operationRow(id, path, origin) {
  if (operationRows.has(id)) return operationRows.get(id);
  if (operationRows.size >= 256) throw new Error("The supported operation display bound was reached. Relaunch the workspace.");
  const region = operationRegion();
  const article = node("article"); article.setAttribute("data-operation-id", id);
  const status = node("p"); status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite"); status.setAttribute("aria-atomic", "true"); status.setAttribute("data-operation-status", ""); status.tabIndex = -1; status.setAttribute("aria-label", `Operation ${id} status`);
  const error = node("div"); error.hidden = true; error.setAttribute("role", "alert"); error.setAttribute("data-operation-error", "");
  const row = { id, path, origin, article, status, error, generation: 0, polling: false, unknown: false, terminal: false, cancelRequested: false, cancellationPending: false, reviewing: false, last: null };
  /** Guard action activation without disabling a focused native control. */
  const operationAction = (label, action) => {
    const control = node("button", label); control.type = "button"; control.setAttribute("aria-disabled", "false");
    control.addEventListener("click", async () => { if (operationCurrent(row) && control.getAttribute("aria-disabled") !== "true") await action(); });
    return control;
  };
  row.cancel = operationAction("Cancel pending operation", async () => {
    row.cancellationPending = true; row.generation++;
    row.status.textContent = "Requesting cancellation; awaiting acknowledgment."; refreshOperationActions(row);
    try {
      const value = await api(`/operations/${encodeURIComponent(id)}/cancellation`, "POST", {});
      if (!operationCurrent(row)) return;
      if (value.cancel_requested !== true) throw new Error("Cancellation could not be verified. Check the operation status.");
      row.cancelRequested = true; acceptOperation(row, value);
    } catch (failure) { if (operationCurrent(row)) operationUnknown(row, failure); }
    finally { row.cancellationPending = false; if (operationCurrent(row)) refreshOperationActions(row); }
  });
  row.check = operationAction("Check operation status", async () => { if (!row.polling) await pollOperation(row, true); }); row.check.hidden = true;
  row.review = operationAction("Review prepared write", async () => {
    if (!operationCurrent(row) || row.last?.state !== "succeeded" || row.polling || row.reviewing) return;
    row.reviewing = true; refreshOperationActions(row);
    try {
      if (!await allowViewChange()) return;
      const reviewOrigin = pending; const result = row.last.result;
      const opened = await preview(result.preview || result.report_preview, exportDownloadContext(path, id), () => operationCurrent(row) && reviewOrigin === pending && !document.querySelector("dialog[open]"));
      if (opened && operationCurrent(row)) { row.error.hidden = true; row.status.textContent = "Operation succeeded. Preparation is ready for review; no write has been confirmed."; }
    } catch (failure) { if (operationCurrent(row)) operationPreviewFailure(row, failure); }
    finally { row.reviewing = false; if (operationCurrent(row)) refreshOperationActions(row); }
  }); row.review.hidden = true;
  article.append(node("h2", `Operation ${id}`), status, error, row.cancel, row.check, row.review); region.append(article);
  operationRows.set(id, row); return row;
}

/** Removed rows and stopped sessions cannot publish a response or open a preview. */
function operationCurrent(row) {
  return !stopped && row.article.isConnected && operationRows.get(row.id) === row;
}

/** Preserve a focused action while making unavailable operations inert to activation. */
function refreshOperationActions(row) {
  const unavailable = row.terminal || row.unknown || row.cancelRequested || row.cancellationPending || stopped;
  row.cancel.setAttribute("aria-disabled", String(unavailable));
  row.cancel.hidden = row.terminal && document.activeElement !== row.cancel;
  row.check.setAttribute("aria-disabled", String(row.polling || row.terminal || stopped));
  row.check.hidden = !row.unknown && document.activeElement !== row.check;
  row.review.hidden = row.last?.state !== "succeeded";
  row.review.setAttribute("aria-disabled", String(row.polling || row.reviewing || stopped));
}

/** Recover unverified work by GET; a late action error cannot erase verified terminal state. */
function operationUnknown(row, failure) {
  row.error.textContent = failure instanceof Error ? failure.message : "The operation could not be verified."; row.error.hidden = false;
  if (!row.terminal) {
    row.unknown = true;
    row.status.textContent = `${row.cancelRequested ? "Cancellation was requested. " : ""}Operation outcome is unknown. Check operation status before retrying.`;
  }
  refreshOperationActions(row);
}

/** A preview-read failure cannot erase a verified successful preparation state. */
function operationPreviewFailure(row, failure) {
  row.unknown = false;
  row.status.textContent = "Operation succeeded. The prepared write could not be loaded; review it again before confirming.";
  row.error.textContent = failure instanceof Error ? failure.message : "The prepared write could not be read."; row.error.hidden = false;
  refreshOperationActions(row);
}

/** Accept each declared preparation family and report capture facts without inventing byte publication. */
function acceptOperation(row, value) {
  const expectedKind = { "/conversions": "conversion", "/applicability/analyses": "applicability-analysis", "/mapping/builds": "mapping-build", "/exports": "export", "/project/bundle-exports": "export", "/project/source-bundle-exports": "export" }[row.path];
  if (value?.operation_id !== row.id || !expectedKind || value.kind !== expectedKind ||
      !["pending", "running", "succeeded", "failed", "cancelled"].includes(value.state) || typeof value.cancel_requested !== "boolean" ||
      !Number.isFinite(Date.parse(value.created_at)) || !Number.isFinite(Date.parse(value.updated_at))) throw new Error("The operation returned an unsupported state or identity. Check its status before continuing.");
  if (row.terminal) return;
  let progress = "Progress is indeterminate.";
  if (value.progress !== undefined && value.progress !== null) {
    if (!Number.isSafeInteger(value.progress.completed_items) || value.progress.completed_items < 0 || !Number.isSafeInteger(value.progress.total_items) || value.progress.total_items < 0) throw new Error("The operation returned unsupported progress facts. Check its status before continuing.");
    progress = value.progress.completed_items > value.progress.total_items ? "Reported progress counters do not reconcile; completion is indeterminate." : `Captured registrations: ${value.progress.completed_items} of ${value.progress.total_items} reported.`;
  }
  if (value.state === "succeeded" && !(value.result?.preview?.preview_id || value.result?.report_preview?.preview_id)) throw new Error("The operation succeeded without a verified prepared-write reference. Check its status before continuing.");
  row.last = value; row.cancelRequested ||= value.cancel_requested; row.unknown = false; row.error.hidden = true;
  row.terminal = !["pending", "running"].includes(value.state);
  const text = value.state === "succeeded" ? "Operation succeeded. Preparation is ready for review; no write has been confirmed." :
    value.state === "failed" ? `Operation failed. ${value.error?.message || "No prepared write is available."}` :
    value.state === "cancelled" ? "Operation cancelled. No prepared write is available." :
    `Operation ${value.state}. ${row.cancelRequested ? "Cancellation requested; awaiting terminal state. " : ""}${progress}`;
  if (row.status.textContent !== text) row.status.textContent = text;
  refreshOperationActions(row);
}

/** Poll known preparation with bounded GET recovery; newer previews retire source auto-opening. */
async function pollOperation(row, checkImmediately = false) {
  if (!operationCurrent(row) || row.polling || row.terminal) return;
  row.polling = true; refreshOperationActions(row);
  try {
    for (let attempts = 0; attempts < 120 && operationCurrent(row) && !row.terminal; attempts++) {
      if (!checkImmediately || attempts > 0) await new Promise(resolve => setTimeout(resolve, 500));
      if (!operationCurrent(row)) return;
      const generation = row.generation;
      let value;
      try { value = await api(`/operations/${encodeURIComponent(row.id)}`); }
      catch (failure) { if (!operationCurrent(row)) return; if (generation !== row.generation) continue; throw failure; }
      if (!operationCurrent(row)) return;
      if (generation !== row.generation) continue;
      acceptOperation(row, value);
    }
    if (operationCurrent(row) && !row.terminal) operationUnknown(row, new Error("Polling reached its supported read bound. Check operation status to continue."));
    if (operationCurrent(row) && row.last?.state === "succeeded" && row.origin === pending && (row.autoReviewGeneration === undefined || row.autoReviewGeneration === previewGeneration) && (!row.autoReviewCurrent || row.autoReviewCurrent()) && !document.querySelector("dialog[open]")) {
      await preview(row.last.result.preview || row.last.result.report_preview, exportDownloadContext(row.path, row.id), () => operationCurrent(row) && row.origin === pending && (!row.autoReviewCurrent || row.autoReviewCurrent()) && !document.querySelector("dialog[open]"));
    }
  } catch (failure) {
    if (operationCurrent(row)) {
      if (row.last?.state === "succeeded" && row.terminal) operationPreviewFailure(row, failure); else operationUnknown(row, failure);
    }
  }
  finally { row.polling = false; if (operationCurrent(row)) refreshOperationActions(row); }
}

/** Recover preparation with its original body/key; source confirmation and newer-preview ownership stay separate. */
async function effect(path, method, request, rawBody, requestIsCurrent, requestedIndexSchema, sourcePreviewOwner = previewGeneration) {
  const key = crypto.randomUUID(); const origin = pending; const sourcePreparation = ["/project/source-bundle-exports","/project/source-bundle-imports"].includes(path); const previewOwner = sourcePreviewOwner; let row; let recovery; let sending = false; let dispatched = false;
  /** Replay only an unacknowledged request; once acknowledged, query its existing operation. */
  const send = async () => {
    if (stopped) return;
    if (row) { await pollOperation(row, true); return; }
    let value;
    try {
      value = await api(path, method, request, key, rawBody, !requestIsCurrent && rawBody === undefined ? undefined : () => {
        const current = dispatched || (!sourcePreparation || previewOwner === previewGeneration) && (!requestIsCurrent || requestIsCurrent());
        if (current) dispatched = true;
        return current;
      });
    } catch (failure) {
      if (!dispatched && (requestIsCurrent && !requestIsCurrent() || sourcePreparation && previewOwner !== previewGeneration)) { removePendingRequest(recovery); return; }
      throw failure;
    }
    if (stopped) return;
    if (value?.operation_id !== undefined || value?.state !== undefined) {
      if (typeof value.operation_id !== "string" || !/^op_[0-9a-z]{12,80}$/.test(value.operation_id)) {
        const unsupported = new Error("The operation returned an unsupported identity. Relaunch the workspace before continuing."); unsupported.details = { retryable: false }; throw unsupported;
      }
      row = operationRow(value.operation_id, path, origin); row.autoReviewCurrent = requestIsCurrent; if (sourcePreparation) row.autoReviewGeneration = previewOwner; removePendingRequest(recovery, row.status);
      try {
        acceptOperation(row, value);
        if (!row.terminal) await pollOperation(row);
        else if (row.last?.state === "succeeded" && origin === pending && (!sourcePreparation || previewOwner === previewGeneration) && (!requestIsCurrent || requestIsCurrent()) && !document.querySelector("dialog[open]")) await preview(row.last.result.preview || row.last.result.report_preview, exportDownloadContext(path, row.id), () => operationCurrent(row) && origin === pending && (!requestIsCurrent || requestIsCurrent()) && !document.querySelector("dialog[open]"));
      } catch (failure) {
        if (operationCurrent(row)) {
          if (row.last?.state === "succeeded" && row.terminal) operationPreviewFailure(row, failure); else operationUnknown(row, failure);
        }
      }
      return;
    }
    if (path === "/project/bundle-imports") {
      const checked = await checkedBundleImportPreview(value, requestedIndexSchema);
      recovery.prepared = checked.preview; recovery.reviewContext = checked;
    } else if (path === "/project/source-bundle-imports") {
      const checked = await checkedSourceImportPreview(value, requestedIndexSchema);
      recovery.prepared = checked.preview; recovery.reviewContext = checked;
      sourceRestoreRow(checked.preview.operation_id, checked.preview);
    } else recovery.prepared = value.preview || value.report_preview;
    if (!recovery.prepared?.preview_id) throw new Error("The operation did not return a prepared write.");
    const transferFocus = document.activeElement === recovery.retry;
    recovery.article.hidden = false; recovery.retry.hidden = true; recovery.review.hidden = false; recovery.error.hidden = true;
    recovery.status.textContent = "Preparation is ready for review; no write has been confirmed.";
    if (transferFocus && recovery.status.isConnected) recovery.status.focus();
    const opened = (!sourcePreparation || previewOwner === previewGeneration) && await preview(recovery.prepared, undefined, () => !stopped && origin === pending && (!requestIsCurrent || requestIsCurrent()) && !document.querySelector("dialog[open]"), recovery.reviewContext);
    if (opened) removePendingRequest(recovery);
  };
  /** Keep background failures local while preserving the original request's replay identity. */
  const attempt = async () => {
    if (sending || stopped) return;
    sending = true;
    try { await send(); } catch (failure) {
      if (stopped) return;
      pendingRequestFailure(recovery, failure);
      if (origin === pending && (!sourcePreparation || previewOwner === previewGeneration) && (!requestIsCurrent || requestIsCurrent()) && !recovery.busy) {
        showError(failure);
        if (!row && !recovery.prepared && failure.details?.retryable !== false) element("error").append(button("Retry the same request", attempt));
      }
    } finally { sending = false; }
  };
  try { recovery = pendingRequestRow(key, path, attempt); } catch (failure) { if (!stopped) showError(failure); return; }
  await attempt();
}

/** Serialize native close processing before the shared preview dialog may be reused. */
function previewCloseLifecycle(dialog, close = false) {
  if (!previewClosePending) {
    const waiting = new Promise(resolve => {
      /** Release only this queued close's barrier before any successor preview is installed. */
      function finishPreviewClose() {
        if (previewClosePending === waiting) previewClosePending = null;
        resolve();
      }
      dialog.addEventListener("close", finishPreviewClose, {once:true});
    });
    previewClosePending = waiting;
  }
  const waiting = previewClosePending;
  if (close) dialog.close();
  return waiting;
}

/** Review exact single-file receipts or delegate the isolated source batch dialog with shared close ownership. */
async function preview(proposed, exportOperation, isCurrent = () => !stopped, reviewContext) {
  if (!isCurrent()) return false;
  if (reviewContext?.family === "source-restore") return previewSourceRestore(reviewContext, isCurrent);
  if(!proposed?.preview_id)throw new Error("The operation did not return a prepared write.");
  const sequence = ++previewGeneration;
  const current = await api(`/effects/previews/${encodeURIComponent(proposed.preview_id)}`);
  if (previewClosePending) await previewClosePending;
  if (!isCurrent() || sequence !== previewGeneration) return false;
  if (reviewContext && (reviewContext.preview.preview_id !== current.preview_id ||
      reviewContext.replacement.proposed_index_sha256 !== current.exact_bytes_sha256 ||
      current.operation_type !== "workspace-index-update" || current.target.path !== "forge.workspace.json"))
    throw new Error("The index replacement no longer matches the exact server preview. Prepare it again.");
  const dialog = element("preview-dialog"); const content = element("preview-content");
  let viewOwner = pending;
  let closingConfirmed = false;
  /** New previews, navigation, dismissal and shutdown revoke this receipt's UI ownership. */
  const previewCurrent = (requireOpen = true) => !stopped && sequence === previewGeneration && viewOwner === pending &&
    dialog.isConnected && (requireOpen ? dialog.open : !document.querySelector("dialog[open]"));
  /** Native Escape invalidates immediately; queued close events belong to their original preview. */
  function invalidatePreview(event) {
    if (event.type === "cancel") previewCloseLifecycle(dialog);
    if (event.type === "close" && dialog.open) return;
    if (event.type === "close") { dialog.removeEventListener("cancel", invalidatePreview); dialog.removeEventListener("close", invalidatePreview); }
    if (sequence === previewGeneration && (!closingConfirmed || event.type === "cancel")) previewGeneration++;
  }
  dialog.addEventListener("cancel", invalidatePreview);
  dialog.addEventListener("close", invalidatePreview);
  dialog.querySelector("[role=alert]")?.remove();
  content.replaceChildren(Object.assign(node("h2", "Review proposed write"),{id:"preview-title"}),node("p", `${current.target.status}: ${current.target.path}`),node("p", current.semantic_summary),
    node("p", `Validation: ${current.validation.state}. Target version: ${current.target_version}`),node("p", `Current hash: ${current.base_sha256 || "new file"}`),node("p", `Proposed hash: ${current.exact_bytes_sha256}`),node("p", `Receipt expires: ${current.receipt.expires_at}`),
    table("Bound input hashes",[["Resource","resource_id"],["SHA-256","sha256"]],current.input_hashes),Object.assign(node("pre",current.diff_text),{tabIndex:0}));
  if(current.diff_truncated) content.append(node("p","The text diff reached its display bound. The hash binds the complete proposed bytes."));
  if (reviewContext) appendBundleReplacement(content, reviewContext.replacement);
  const key = crypto.randomUUID();
  /** Download only the prepared family; JSON metadata/source artifacts bind media, cap, hash and view ownership. */
  async function downloadCommittedExport() {
    const metadata = typeof exportOperation === "object" && exportOperation?.family === "metadata-bundle";
    const source = typeof exportOperation === "object" && exportOperation?.family === "source-bundle";
    const json = metadata || source;
    const id = json ? exportOperation.operation_id : exportOperation;
    const owner = pending;
    if (json && (stopped || (source ? !sourceBundleEffectsSupported() : !bundleEffectsSupported()))) return;
    try {
    const path = source ? `/project/source-bundle-exports/${encodeURIComponent(id)}/download` : metadata ? `/project/bundle-exports/${encodeURIComponent(id)}/download` : `/exports/${encodeURIComponent(id)}/download`;
    const response=await fetch(`${apiPrefix}${path}`,{headers:{Authorization:`Bearer ${capability}`},cache:"no-store",credentials:"omit",redirect:"error",referrerPolicy:"no-referrer"});
    if (json && (stopped || owner !== pending)) return;
    if(!response.ok)throw new Error("The committed export is no longer available or its bytes changed.");
    const blob=await response.blob();if (json && (stopped || owner !== pending)) return;
    if(blob.size>(source?1048429:metadata?1024*1024:4*1024*1024))throw new Error("The export exceeds the download bound.");
    if (json) {
      if (response.headers.get("Content-Type")?.split(";",1)[0].trim().toLowerCase() !== "application/json") throw new Error("The committed JSON bundle has an unsupported media type.");
      const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", await blob.arrayBuffer()));
      const hash = Array.from(digest, byte => byte.toString(16).padStart(2,"0")).join("");
      if (stopped || owner !== pending) return;
      if (hash !== current.exact_bytes_sha256) throw new Error("The committed JSON bundle bytes do not match the prepared hash.");
      if (stopped || owner !== pending || !element("view").isConnected) return;
    }
    const url=URL.createObjectURL(blob);const link=node("a",source?"Download exact source bundle":metadata?"Download metadata bundle":"Download report");link.href=url;link.download=source?"forge-workspace-index-and-source-content.json":metadata?"forge-workspace-index-and-hashes.json":"forge-redacted-report.html";document.body.append(link);link.click();link.remove();
    if (json) { metadataDownloadURLs.add(url);setTimeout(()=>{if(metadataDownloadURLs.delete(url))URL.revokeObjectURL(url);},1000); }
    else setTimeout(()=>URL.revokeObjectURL(url),1000);
    } catch (failure) { if (json && (stopped || owner !== pending)) return; throw failure; }
  }
  /** Await native return-focus processing before selecting the still-owned saved result target. */
  async function closeConfirmedPreview() {
    closingConfirmed = true;
    await previewCloseLifecycle(dialog, true);
  }
  /** Dismissal revokes the old write immediately and waits for its native return-focus lifecycle. */
  function dismissPreview() {
    if (sequence === previewGeneration) previewGeneration++;
    return previewCloseLifecycle(dialog, true);
  }
  /** One receipt/key may publish saved UI only while its preview and destination remain current. */
  async function confirmWrite() {
    if (!previewCurrent()) return;
    dialog.querySelector("[role=alert]")?.remove();
    try {
      const operation = await api("/effects/commits","POST",{receipt:current.receipt.token,observed_version:current.target_version,confirmed:true},key);
      const observed = await api(`/operations/${encodeURIComponent(operation.operation_id)}`);
      if (!previewCurrent()) return;
      if(observed.state !== "succeeded") throw new Error(observed.error?.message || "The write has not completed.");
      const refresh = renderView(() => !stopped && sequence === previewGeneration && dialog.open);
      viewOwner = pending;
      const refreshed = await refresh;
      if (!previewCurrent()) return;
      const pageError = element("view").querySelector("[data-page-error]");
      const refreshError = dialog.querySelector("[role=alert]")?.textContent || (!pageError?.hidden && pageError?.textContent) || (!element("error").hidden && element("error").textContent) || "The view could not be loaded.";
      if (exportOperation) element("view").prepend(button(exportOperation?.family === "source-bundle" ? "Download committed source bundle" : typeof exportOperation === "object" ? "Download committed metadata bundle" : "Download committed redacted report", downloadCommittedExport));
      dirty = false;
      await closeConfirmedPreview();
      if (!previewCurrent(false)) return;
      element("status").textContent = `Saved ${observed.result.target_path}.`;
      if (refreshed) focusViewTitle();
      else showError(new Error(`The write was saved, but the view could not be refreshed. ${refreshError}`));
    } catch (error) { if (previewCurrent()) showError(error); }
  }
  content.append(button("Keep editing", dismissPreview), button("Confirm this exact write", confirmWrite));
  dialog.showModal();content.querySelector("h2").tabIndex=-1;content.querySelector("h2").focus();return true;
}
/** List closed registration roles for an explicit index version; /1 retains its original families. */
function workspaceRoles(version) {
  const original = ["policy-source", "oscal-catalog-artifact", "oscal-component-artifact", "mapping-collection", "applicability-manifest", "applicability-report", "trace-report"];
  if (version === "forge.workspace/1") return original;
  if (version === "forge.workspace/2") return [...original, "lifecycle-record", "lifecycle-source", "oscal-profile-artifact", "oscal-ssp-artifact", "framework-impact-manifest", "successor-map", "framework-impact-report", "framework-impact-dispositions"];
  return [];
}

/** Prepare bounded registrations or an explicit version migration through ordinary exact-write confirmation. */
function resourceActions(resources) {
  const region = node("section");region.append(node("h2","Project files"));
  region.append(button("Validate registered resources",async () => { const report = await api("/validation/runs","POST",{scope:"all"});element("status").textContent = `Validation: ${report.state}. ${report.error_count} errors.`; }));
  if(readOnly) {region.append(node("p","This session is read-only."));return region;}
  const roles = workspaceRoles(apiMajor === 2 ? "forge.workspace/2" : "forge.workspace/1").map(value=>[value,value.replaceAll("-"," ")]);
  const register = node("form");register.append(node("h3","Register an existing project file"));
  const role = field(register,"Resource role","select",roles);
  const path = field(register,"Project-relative file path");
  const key = field(register,"Stable resource key");
  const indexVersion = apiMajor === 2 ? field(register,"Index version for this registration","select",[["preserve","Preserve current index version"],["forge.workspace/2","Explicitly use workspace index /2"]]) : null;
  if (apiMajor === 2) register.append(node("p","Lifecycle and framework-impact roles require an explicit /2 index. Admission profiles describe structure or fingerprints; they do not establish current freshness, dependency closure or reviewer authority."));
  register.append(button("Preview registration",async () => {
    if(!register.reportValidity())return;
    const request = {role:role.value,path:path.value,key:key.value};
    if(indexVersion?.value === "forge.workspace/2") request.index_schema_version = indexVersion.value;
    await effect("/resources/register","POST",request);
  }));
  if (apiMajor === 2) register.append(button("Preview migration to workspace index /2",async () => {
    await effect("/resources/register","POST",{migration:{from:"forge.workspace/1",to:"forge.workspace/2"}});
  }));
  register.addEventListener("submit",event=>event.preventDefault());
  const upload = node("form");upload.append(node("h3","Upload a local file"));
  const file = field(upload,"Choose a file","file"); const uploadRole = field(upload,"Uploaded resource role","select",roles);const target = field(upload,"Destination path within project");
  upload.append(button("Preview upload",async () => {if(!upload.reportValidity())return;const selected=file.files[0];if(selected.size>10*1024*1024)throw new Error("Files must be at most 10 MiB.");const bytes=new Uint8Array(await selected.arrayBuffer());let binary="";for(let i=0;i<bytes.length;i+=8192)binary+=String.fromCharCode(...bytes.subarray(i,i+8192));await effect("/resources/upload","POST",{role:uploadRole.value,target_path:target.value,filename:selected.name,content_base64:btoa(binary)});}));
  upload.append(node("p","After confirming the upload, register its project-relative path separately."));upload.addEventListener("submit",event=>event.preventDefault());
  const conversion = node("form");conversion.append(node("h3","Convert a supplied Markdown policy"));
  const policies=resources.filter(r=>r.role==="policy-source");
  if(policies.length) {const source=field(conversion,"Policy source","select",policies.map(r=>[r.resource_id,r.key]));const kind=field(conversion,"Output model","select",[["oscal-catalog","OSCAL Catalog"],["oscal-component-definition","OSCAL Component Definition"]]);const output=field(conversion,"Output project-relative path");conversion.append(button("Prepare conversion",async()=>{if(conversion.reportValidity())await effect("/conversions","POST",{source_resource_id:source.value,output_kind:kind.value,target_path:output.value});}));}
  else conversion.append(node("p","Register a Markdown policy before preparing a conversion."));
  conversion.addEventListener("submit",event=>event.preventDefault());region.append(register,upload,conversion);return region;
}
async function draftEditor(kind) {
  let draft;
  try {draft=await api(`/${kind}/draft`);} catch (error) {if (error.details?.code === "not-found") return initializeForm(kind); throw error;}
  const form=node("form");
  form.append(node("h2",kind==="mapping"?"Explicit mapping decisions":"Explicit applicability decisions"),node("p","Edit the complete decision document. Supply the reviewer key, review time, state or relationship, and rationale explicitly. Reviewer metadata is asserted provenance."));
  const input=field(form,"Decision manifest (JSON)","textarea");input.value=JSON.stringify(draft.manifest,null,2);input.readOnly=readOnly;
  if(!readOnly) {
    try {
      if(kind==="applicability")form.append(await applicabilityDecisionForm(input),await applicabilityMappingForm(input));
      else form.append(await mappingDecisionForm(input));
    } catch(error) {form.append(node("p",`Guided inventory is unavailable: ${error.message} The complete decision document remains available for explicit repair.`));}
  }
  form.append(button("Validate decisions",async()=>{const report=await api(`/${kind}/draft/validation`,"POST",{manifest:JSON.parse(input.value)});element("status").textContent=`Decision validation: ${report.state}. ${report.error_count} errors.`;}));
  if(!readOnly) {
    form.append(button("Preview decision changes",()=>effect(`/${kind}/draft`,"PUT",{manifest:JSON.parse(input.value),observed_version:draft.version})));
    form.append(button(kind==="mapping"?"Rebuild committed mapping collection":"Analyze committed scope decisions",()=>effect(kind==="mapping"?"/mapping/builds":"/applicability/analyses","POST",{})));
  }
  form.addEventListener("submit",event=>event.preventDefault());return form;
}
async function applicabilityDecisionForm(input) {
  const section=node("section");section.append(node("h3","Record one reviewed control decision"));
  const controls=await collection("/applicability/controls");
  const control=field(section,"Control to review","select",[["","Choose a control"],...controls.map(row=>[row.control_id,row.control_id])]);
  const state=field(section,"Explicit decision","select",[["","Choose a state"],...["applicable","not-applicable","deferred","under-review"].map(value=>[value,value.replaceAll("-"," ")])]);
  const reviewer=field(section,"Decision reviewer key");const name=field(section,"Decision reviewer name");const time=field(section,"Decision review time (RFC3339)");const rationale=field(section,"Decision rationale");const revisit=field(section,"Deferred revisit date","date");revisit.required=false;
  state.addEventListener("change",()=>{revisit.required=state.value==="deferred";});
  section.append(button("Apply decision to unsaved manifest",()=>{
    for(const field of section.querySelectorAll("input,select"))if(!field.reportValidity())return;
    const manifest=JSON.parse(input.value);const previous=manifest.reviewers.find(row=>row.key===reviewer.value);
    if(previous&&previous.name!==name.value)throw new Error("This reviewer key already names a different asserted reviewer. Review the manifest explicitly before changing that record.");
    if(!previous)manifest.reviewers.push({key:reviewer.value,type:"person",name:name.value});
    const decision={control_id:control.value,state:state.value,reviewer_key:reviewer.value,reviewed_at:time.value,rationale:rationale.value};
    if(state.value==="deferred")decision.revisit_date=revisit.value;
    const index=manifest.decisions.findIndex(row=>row.control_id===control.value);
    if(index<0)manifest.decisions.push(decision);else manifest.decisions[index]=decision;
    input.value=JSON.stringify(manifest,null,2);dirty=true;input.focus();element("status").textContent="Decision added to the unsaved manifest. Validate and preview before saving.";
  }));return section;
}

async function mappingDecisionForm(input) {
  const section=node("section");section.append(node("h3","Edit a reviewed relationship"));
  const manifest=JSON.parse(input.value);
  const selected=field(section,"Relationship to review","select",[["","Choose a relationship"],...manifest.mapping.maps.map(row=>[row.key,row.key])]);
  const context=node("p");section.append(context);
  const relationship=field(section,"New explicit relationship","select",[["","Choose a relationship"],...["equivalent-to","equal-to","subset-of","superset-of","intersects-with","no-relationship"].map(value=>[value,value.replaceAll("-"," ")])]);
  const reviewer=field(section,"Mapping reviewer","select",[["","Choose a supplied reviewer"],...manifest.reviewers.map(row=>[row.key,`${row.key}: ${row.name}`])]);
  const time=field(section,"Mapping review time (RFC3339)");const rationale=field(section,"Mapping review rationale");
  selected.addEventListener("change",()=>{
    const current=JSON.parse(input.value).mapping.maps.find(row=>row.key===selected.value);if(!current)return;
    context.textContent=`Sources: ${current.sources.map(row=>`${row.type} ${row.id_ref}`).join(", ")}. Targets: ${current.targets.map(row=>`${row.type} ${row.id_ref}`).join(", ")}.`;
    relationship.value=current.relationship;reviewer.value=current.reviewer_key;time.value=current.reviewed_at;rationale.value=current.rationale;
  });
  section.append(button("Apply relationship to unsaved manifest",()=>{
    for(const field of section.querySelectorAll("input,select"))if(!field.reportValidity())return;
    const manifest=JSON.parse(input.value);const current=manifest.mapping.maps.find(row=>row.key===selected.value);
    if(!current)throw new Error("The selected relationship changed in the unsaved manifest. Reload its selection.");
    Object.assign(current,{relationship:relationship.value,reviewer_key:reviewer.value,reviewed_at:time.value,rationale:rationale.value});
    input.value=JSON.stringify(manifest,null,2);dirty=true;input.focus();element("status").textContent="Relationship updated in memory. Validate and preview before saving.";
  }));
  const container=node("div");container.append(section,await mappingCreationForm(input));return container;
}

async function mappingCreationForm(input) {
  const section=node("section");section.append(node("h3","Add an explicit reviewed relationship"));
  const manifest=JSON.parse(input.value);const subjects=await collection("/mapping/subjects");
  const key=field(section,"New relationship key");
  const options=side=>[["","Choose a subject"],...subjects.filter(row=>row.side===side&&(manifest.mapping.scope!=="control-only"||row.statement_count===0)).map(row=>[JSON.stringify({type:row.statement_count===0?"control":"statement",id_ref:row.label}),row.label])];
  const source=field(section,"New relationship policy subject","select",options("policy"));
  const target=field(section,"New relationship framework subject","select",options("framework"));
  const relationship=field(section,"New relationship type","select",[["","Choose a relationship"],...["equivalent-to","equal-to","subset-of","superset-of","intersects-with","no-relationship"].map(value=>[value,value.replaceAll("-"," ")])]);
  const reviewer=field(section,"New relationship reviewer","select",[["","Choose a supplied reviewer"],...manifest.reviewers.map(row=>[row.key,`${row.key}: ${row.name}`])]);
  const time=field(section,"New relationship review time (RFC3339)");const rationale=field(section,"New relationship rationale");
  section.append(button("Add relationship to unsaved manifest",()=>{
    for(const control of section.querySelectorAll("input,select"))if(!control.reportValidity())return;
    const current=JSON.parse(input.value);
    if(current.mapping.maps.some(row=>row.key===key.value))throw new Error("This relationship key already exists. Choose it in the review editor to update it.");
    current.mapping.maps.push({key:key.value,relationship:relationship.value,sources:[JSON.parse(source.value)],targets:[JSON.parse(target.value)],reviewer_key:reviewer.value,reviewed_at:time.value,rationale:rationale.value});
    input.value=JSON.stringify(current,null,2);dirty=true;input.focus();element("status").textContent="Relationship added in memory. Validate and preview before saving.";
  }));return section;
}
function relativeProjectReference(manifest, resource) {
  const parent=manifest.split("/").slice(0,-1);const target=resource.split("/");let common=0;
  while(common<parent.length&&common<target.length&&parent[common]===target[common])common++;
  return [...Array(parent.length-common).fill(".."),...target.slice(common)].join("/");
}
async function applicabilityMappingForm(input) {
  const section=node("section");section.append(node("h3","Link a reviewed mapping collection"));
  const resources=await collection("/resources");const manifests=resources.filter(row=>row.role==="applicability-manifest");
  if(manifests.length!==1)return section;
  const collections=resources.filter(row=>row.role==="mapping-collection");
  const selected=field(section,"Reviewed mapping collection","select",[["","Choose a registered collection"],...collections.map(row=>[row.path,`${row.key} · ${row.path}`])]);
  section.append(node("p","Choose a built OSCAL mapping collection. Full domain validation checks its exact framework identity and reviewed relationships before any write."));
  section.append(button("Link collection to unsaved scope",()=>{
    if(!selected.reportValidity())return;
    const manifest=JSON.parse(input.value);const reference=relativeProjectReference(manifests[0].path,selected.value);
    if(manifest.mapping_collections.includes(reference))throw new Error("This collection is already linked in the unsaved scope document.");
    manifest.mapping_collections.push(reference);input.value=JSON.stringify(manifest,null,2);dirty=true;input.focus();element("status").textContent="Collection linked in memory. Validate and preview scope changes, then analyze committed decisions.";
  }));return section;
}

function evidenceList(rows) {
  const list=node("section");list.append(node("h2","Review evidence"));
  for(const row of rows)for(const anchor of row.evidence_refs||(row.provenance_ref?[row.provenance_ref]:[]))list.append(button(`Inspect ${row.control_id||row.label||row.reason_code}`,()=>showProvenance(anchor)));
  return list;
}
/** Inspect/Trace use the same discard gate; failed or superseded reads keep the form. */
async function showProvenance(anchor) {
  if (!await allowViewChange()) return false;
  invalidateBundlePanels();
  const sequence = ++pending;
  element("view").inert = true;
  element("refresh").disabled = true;
  element("status").textContent = "Loading provenance references…";
  try {
    const entries=await collection(`/provenance/entries?anchor=${encodeURIComponent(anchor)}`);
    const region=node("section");region.append(node("h2","Provenance references"),node("p","References describe supplied records. They do not authenticate the person or establish independent review."));
    for(const entry of entries) {
      const item=node("article");item.append(node("h3",entry.label),node("p",`${entry.kind} · ${entry.fingerprint||""}`));
      for(const reference of entry.refs)item.append(button(`Follow ${reference.kind}`,()=>showProvenance(reference.id)));
      for(const id of entry.excerpt_refs||[]) {
        const open=button("Read bounded source excerpt",async()=>{
          const excerpt=await api(`/provenance/excerpts/${encodeURIComponent(id)}`);
          const region=node("section");const heading=node("h4",`Source lines ${excerpt.start_line}–${excerpt.end_line}`);heading.tabIndex=-1;
          region.append(heading,node("p",`SHA-256 ${excerpt.sha256}${excerpt.truncated?" · Truncated":""}`),Object.assign(node("pre",excerpt.text),{tabIndex:0}),button("Close source excerpt",()=>{region.remove();open.focus();}));
          item.append(region);heading.focus();
        });item.append(open);
      }
      region.append(item);
    }
    if(!entries.length)region.append(node("p","No verified references are available for this anchor."));
    if(sequence !== pending)return;
    dirty = false;
    element("error").hidden = true;
    element("view").replaceChildren(region);element("view-title").textContent="Provenance";
    element("status").textContent="Provenance references loaded.";
    focusViewTitle();
  } catch(error) {
    if(sequence === pending) {element("status").textContent="The provenance references could not be loaded.";throw error;}
  } finally {
    if(sequence === pending) {element("view").inert=false;element("refresh").disabled=false;}
  }
}
/** Retire installed metadata reads and acknowledgments before a view epoch changes. */
function invalidateBundlePanels() {
  for (const url of metadataDownloadURLs) { URL.revokeObjectURL(url); metadataDownloadURLs.delete(url); }
  for (const section of element("view").querySelectorAll("[data-bundle-panel]")) bundlePanels.get(section)?.();
}

/** Accept only a supported closed response object, without dropping unknown metadata. */
function bundleClosedObject(value, keys) {
  return value !== null && typeof value === "object" && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
}

/** Validate complete same-version /1 or /2 metadata before retaining a local downloadable bundle. */
function checkedBundlePreview(value) {
  /** Reject unsupported preview metadata before it can become a local file. */
  const fail = () => { throw new Error("The metadata preview returned an unsupported response. Preview metadata again."); };
  if (!bundleClosedObject(value, ["bundle", "snapshot_version", "source_index_present", "included_metadata", "source_content_included"]) ||
      value.source_index_present !== true || value.source_content_included !== false ||
      typeof value.snapshot_version !== "string" || value.snapshot_version.length < 8 || value.snapshot_version.length > 128 ||
      JSON.stringify(value.included_metadata) !== JSON.stringify(["project-label", "resource-keys", "typed-roles", "project-relative-paths", "sha256-fingerprints", "byte-lengths"])) fail();
  const bundle = value.bundle;
  if (!bundleClosedObject(bundle, ["schema_version", "content_profile", "index", "index_sha256", "pins"]) ||
      !(apiMajor === 2 ? ["forge.workspace-index-bundle/1", "forge.workspace-index-bundle/2"] : ["forge.workspace-index-bundle/1"]).includes(bundle.schema_version) || bundle.content_profile !== "index-and-hashes" ||
      typeof bundle.index_sha256 !== "string" || !/^[a-f0-9]{64}$/.test(bundle.index_sha256) ||
      !bundleClosedObject(bundle.index, ["schema_version", "label", "resources"]) || bundle.index.schema_version !== (bundle.schema_version === "forge.workspace-index-bundle/1" ? "forge.workspace/1" : "forge.workspace/2") ||
      typeof bundle.index.label !== "string" || [...bundle.index.label].length < 1 || [...bundle.index.label].length > 200 ||
      !Array.isArray(bundle.index.resources) || bundle.index.resources.length > 1000 ||
      !Array.isArray(bundle.pins) || bundle.pins.length !== bundle.index.resources.length) fail();
  const keys = new Set(); const paths = new Set();
  const roles = workspaceRoles(bundle.index.schema_version);
  for (let index = 0; index < bundle.index.resources.length; index++) {
    const resource = bundle.index.resources[index]; const pin = bundle.pins[index];
    if (!bundleClosedObject(resource, ["key", "role", "path"]) || typeof resource.key !== "string" ||
        !/^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/.test(resource.key) || keys.has(resource.key) || !roles.includes(resource.role) ||
        typeof resource.path !== "string" || resource.path.length > 512 || !/^[A-Za-z0-9][A-Za-z0-9._-]*(\/[A-Za-z0-9][A-Za-z0-9._-]*)*$/.test(resource.path) || paths.has(resource.path) ||
        !bundleClosedObject(pin, ["key", "sha256", "size_bytes"]) || pin.key !== resource.key || typeof pin.sha256 !== "string" ||
        !/^[a-f0-9]{64}$/.test(pin.sha256) || !Number.isSafeInteger(pin.size_bytes) || pin.size_bytes < 0 || pin.size_bytes > 10 * 1024 * 1024) fail();
    keys.add(resource.key); paths.add(resource.path);
  }
  const blob = new Blob([JSON.stringify(bundle)], {type:"application/json"});
  if (blob.size > 1024 * 1024) throw new Error("The complete metadata bundle exceeds the 1 MiB download bound. No partial file was created.");
  return { bundle, blob };
}

/** Reconcile every expected comparison row without treating fingerprints as approval. */
function checkedBundleComparison(value) {
  /** Reject unsupported comparison data without inferring a successful observation. */
  const fail = () => { throw new Error("The fingerprint comparison returned unsupported or unreconciled results. Compare again."); };
  const fields = ["scope", "snapshot_version", "source_index_present", "state", "current_resources", "current_only_resources", "expected_index_matches_current", "expected_resources", "matched_resources", "unregistered_resources", "mismatched_resources", "items", "source_content_included"];
  if (!bundleClosedObject(value, fields) || value.scope !== "registered-fingerprints-only" || value.source_content_included !== false ||
      typeof value.snapshot_version !== "string" || value.snapshot_version.length < 8 || value.snapshot_version.length > 128 ||
      typeof value.source_index_present !== "boolean" || typeof value.expected_index_matches_current !== "boolean" ||
      !["missing-index", "matched", "mismatched"].includes(value.state) || !Array.isArray(value.items)) fail();
  for (const key of ["current_resources", "current_only_resources", "expected_resources", "matched_resources", "unregistered_resources", "mismatched_resources"]) {
    if (!Number.isSafeInteger(value[key]) || value[key] < 0 || value[key] > 1000) fail();
  }
  if (value.matched_resources + value.unregistered_resources + value.mismatched_resources !== value.expected_resources ||
      value.items.length !== value.expected_resources || value.current_only_resources + value.matched_resources + value.mismatched_resources !== value.current_resources ||
      value.state !== (!value.source_index_present ? "missing-index" : value.matched_resources === value.expected_resources ? "matched" : "mismatched") ||
      (!value.source_index_present && (value.current_resources !== 0 || value.expected_index_matches_current !== false))) fail();
  const keys = new Set(); const counts = {matched:0, "not-registered":0, mismatched:0};
  for (const item of value.items) {
    if (!bundleClosedObject(item, ["key", "status", "reason_codes", "observed_resource_validation_state"]) || typeof item.key !== "string" ||
        !/^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/.test(item.key) || keys.has(item.key) || !Object.hasOwn(counts, item.status) ||
        !Array.isArray(item.reason_codes) || item.reason_codes.length > 2 || new Set(item.reason_codes).size !== item.reason_codes.length ||
        item.reason_codes.some(reason => !["registration-not-found", "registration-conflict", "sha256-mismatch", "size-mismatch"].includes(reason)) ||
        !["valid", "stale", "invalid", "not-registered"].includes(item.observed_resource_validation_state)) fail();
    if (item.status === "matched" && (item.reason_codes.length || item.observed_resource_validation_state === "not-registered") ||
        item.status === "not-registered" && (item.reason_codes.length !== 1 || item.reason_codes[0] !== "registration-not-found" || item.observed_resource_validation_state !== "not-registered") ||
        item.status === "mismatched" && (!item.reason_codes.length || item.reason_codes.includes("registration-not-found") || item.observed_resource_validation_state === "not-registered" ||
          item.reason_codes.includes("registration-conflict") && item.reason_codes.length !== 1)) fail();
    counts[item.status]++; keys.add(item.key);
  }
  if (counts.matched !== value.matched_resources || counts["not-registered"] !== value.unregistered_resources || counts.mismatched !== value.mismatched_resources) fail();
  return value;
}

/** Bound a chosen file before reading and wrap its exact bytes without JSON rewriting. */
async function bundleVerificationBody(file) {
  if (!file || !Number.isSafeInteger(file.size) || file.size < 0 || typeof file.arrayBuffer !== "function") throw new Error("Choose one metadata bundle JSON file before comparing.");
  if (file.size > 1024 * 1024 - 11) throw new Error("The chosen file and its 11-byte JSON wrapper must fit the 1 MiB request bound.");
  let bytes;
  try { bytes = await file.arrayBuffer(); } catch { throw new Error("The chosen file could not be read. Choose it again before comparing."); }
  if (!bytes || bytes.byteLength !== file.size || bytes.byteLength > 1024 * 1024 - 11) throw new Error("The chosen file changed or exceeds the supported request bound. Choose it again.");
  const body = new Blob(['{"bundle":', new Uint8Array(bytes), '}'], {type:"application/json"});
  if (body.size !== file.size + 11 || body.size > 1024 * 1024) throw new Error("The chosen file exceeds the complete 1 MiB request bound.");
  return body;
}

/** Keep independent, read-only metadata preview and raw-file comparison in the installed view. */
function metadataBundlePanel() {
  const section = node("section"); section.setAttribute("data-bundle-panel", "");
  const title = node("h2", "Workspace metadata bundle"); title.id = `bundle-${crypto.randomUUID()}`; section.setAttribute("aria-labelledby", title.id);
  const previewStatus = node("p"); previewStatus.setAttribute("role", "status"); previewStatus.setAttribute("aria-live", "polite"); previewStatus.setAttribute("data-bundle-preview-status", "");
  const previewError = node("div"); previewError.setAttribute("role", "alert"); previewError.setAttribute("data-bundle-preview-error", ""); previewError.tabIndex = -1; previewError.hidden = true;
  const metadata = node("div"); metadata.setAttribute("data-bundle-metadata", "");
  const disclosure = node("div");
  const acknowledgment = fieldInput(disclosure, "I understand that labels, resource keys, paths and hashes can reveal project information.", "checkbox"); acknowledgment.required = false;
  const previewButton = node("button", "Preview metadata"); previewButton.type = "button"; previewButton.setAttribute("aria-disabled", "false");
  const download = node("button", "Download metadata bundle"); download.type = "button"; download.setAttribute("aria-disabled", "true"); acknowledgment.setAttribute("aria-disabled", "true");
  const comparisonForm = node("div");
  const file = fieldInput(comparisonForm, "Choose a metadata bundle JSON file", "file"); file.required = false; file.accept = ".json,application/json";
  const compare = node("button", "Compare registered fingerprints"); compare.type = "button"; compare.setAttribute("aria-disabled", "true");
  const comparisonStatus = node("p"); comparisonStatus.setAttribute("role", "status"); comparisonStatus.setAttribute("aria-live", "polite"); comparisonStatus.setAttribute("data-bundle-comparison-status", "");
  const comparisonError = node("div"); comparisonError.setAttribute("role", "alert"); comparisonError.setAttribute("data-bundle-comparison-error", ""); comparisonError.tabIndex = -1; comparisonError.hidden = true;
  const comparison = node("div"); comparison.setAttribute("data-bundle-comparison", "");
  let previewSequence = 0; let comparisonSequence = 0; let previewBusy = false; let comparisonBusy = false; let retained = null;
  const downloadURLs = new Set();
  /** Require this connected section and the current session's visible Trace & Reports view. */
  function installed() { return !stopped && activeView === "Trace & Reports" && section.isConnected && element("view").contains(section) && !element("view").inert; }
  /** Reject results owned by an older local lane, navigation epoch or stopped session. */
  function current(epoch, sequence, lane) { return installed() && epoch === pending && sequence === (lane === "preview" ? previewSequence : comparisonSequence); }
  /** Retain focused native controls while making unavailable actions inert to activation. */
  function refreshActions() {
    const ready = installed() && retained && retained.epoch === pending && retained.sequence === previewSequence;
    previewButton.setAttribute("aria-disabled", String(!installed() || previewBusy));
    acknowledgment.setAttribute("aria-disabled", String(!ready));
    download.setAttribute("aria-disabled", String(!ready || !acknowledgment.checked));
    compare.setAttribute("aria-disabled", String(!installed() || comparisonBusy || !file.files?.length));
    section.setAttribute("aria-busy", String(previewBusy || comparisonBusy));
  }
  /** Revoke each locally created download URL exactly once when retired or dispatched. */
  function revokeDownload(url) { if (downloadURLs.delete(url)) URL.revokeObjectURL(url); }
  /** Clear prior local download authority without changing project edits or focus. */
  function retirePreview() {
    retained = null; acknowledgment.checked = false; metadata.replaceChildren();
    for (const url of [...downloadURLs]) revokeDownload(url);
  }
  /** Retire both read lanes when even a retained view's navigation epoch changes. */
  function invalidatePanel() {
    previewSequence++; comparisonSequence++; previewBusy = false; comparisonBusy = false; retirePreview();
    comparison.replaceChildren(); previewError.hidden = true; comparisonError.hidden = true;
    previewStatus.textContent = "Preview metadata again before downloading.";
    comparisonStatus.textContent = file.files?.length ? "Compare again to observe current registered fingerprints." : ""; refreshActions();
  }
  /** Report a safe current read failure locally, preserving any newer focus owner. */
  function readFailure(error, box, status, invoker) {
    box.textContent = error instanceof Error ? error.message : "The metadata read could not be completed."; box.hidden = false;
    status.textContent = "The read did not complete. Retry explicitly; no project file was written.";
    if (document.activeElement === invoker && !document.querySelector("dialog[open]")) box.focus();
  }
  /** Fetch a complete preview and reset disclosure acknowledgment for this exact local generation. */
  async function loadMetadata() {
    if (!installed() || previewBusy) return;
    const sequence = ++previewSequence; const epoch = pending; previewBusy = true; retirePreview(); previewError.hidden = true;
    previewStatus.textContent = "Reading registered metadata…"; refreshActions();
    try {
      const value = await api("/project/bundle-preview"); if (!current(epoch, sequence, "preview")) return;
      const checked = checkedBundlePreview(value);
      retained = { ...checked, epoch, sequence };
      metadata.append(node("p", `Project: ${checked.bundle.index.label}`), table("Complete registered bundle metadata", [["Key","key"],["Role","role"],["Project-relative path","path"],["SHA-256","sha256"],["Bytes","size_bytes"]], checked.bundle.index.resources.map((resource, index) => ({...resource, ...checked.bundle.pins[index]}))));
      previewStatus.textContent = `Metadata preview: ${checked.bundle.pins.length} registered resources. No project file was written.`;
    } catch (error) { if (current(epoch, sequence, "preview")) readFailure(error, previewError, previewStatus, previewButton); }
    finally { if (sequence === previewSequence) { previewBusy = false; refreshActions(); } }
  }
  /** Acknowledge only a still-owned observed preview, never project approval or a server write. */
  function acknowledgeMetadata() {
    if (!installed() || !retained || retained.epoch !== pending || retained.sequence !== previewSequence) acknowledgment.checked = false;
    refreshActions();
  }
  /** Download the acknowledged local metadata bytes with a fixed safe filename and no effect. */
  function downloadMetadata() {
    if (!installed() || !retained || retained.epoch !== pending || retained.sequence !== previewSequence || !acknowledgment.checked) return;
    const url = URL.createObjectURL(retained.blob); downloadURLs.add(url);
    const link = node("a"); link.href = url; link.download = "forge-workspace-index-and-hashes.json";
    document.body.append(link); link.click(); link.remove();
    previewStatus.textContent = "Metadata download requested. No project file was written.";
    setTimeout(() => revokeDownload(url), 1000);
  }
  /** A new file cancels only obsolete comparison work without retaining raw file content in storage. */
  function fileChanged() {
    comparisonSequence++; comparisonBusy = false; comparisonError.hidden = true; comparison.replaceChildren();
    comparisonStatus.textContent = file.files?.length ? `Chosen file: ${file.files[0].name} (${file.files[0].size} bytes). Compare explicitly.` : "Choose one metadata bundle JSON file."; refreshActions();
  }
  /** Compare exact chosen bytes only against registered captures; each explicit retry is a fresh read. */
  async function compareFile() {
    if (!installed() || comparisonBusy || !file.files?.length) return;
    const selected = file.files[0]; const sequence = ++comparisonSequence; const epoch = pending; comparisonBusy = true;
    comparisonError.hidden = true; comparison.replaceChildren(); comparisonStatus.textContent = "Reading the chosen file for registered fingerprint comparison…"; refreshActions();
    try {
      const body = await bundleVerificationBody(selected); if (!current(epoch, sequence, "comparison")) return;
      const response = await api("/project/bundle-verifications", "POST", undefined, undefined, body, () => current(epoch, sequence, "comparison")); if (!current(epoch, sequence, "comparison")) return;
      const value = checkedBundleComparison(response);
      comparison.append(node("p", `Comparison state: ${value.state}. Whole index matches: ${value.expected_index_matches_current ? "yes" : "no"}.`),
        table("Complete expected fingerprint comparison", [["Key","key"],["Fingerprint","status"],["Reasons","reasons"],["Observed content state","observed_resource_validation_state"]], value.items.map(item => ({...item,reasons:item.reason_codes.join(", ") || "none"}))));
      comparisonStatus.textContent = `Registered fingerprints: ${value.matched_resources} matched, ${value.unregistered_resources} not registered, ${value.mismatched_resources} mismatched of ${value.expected_resources} expected. Current-only registrations: ${value.current_only_resources}.`;
    } catch (error) { if (current(epoch, sequence, "comparison")) readFailure(error, comparisonError, comparisonStatus, compare); }
    finally { if (sequence === comparisonSequence) { comparisonBusy = false; refreshActions(); } }
  }
  previewButton.addEventListener("click", loadMetadata); acknowledgment.addEventListener("change", acknowledgeMetadata);
  download.addEventListener("click", downloadMetadata); file.addEventListener("change", fileChanged); compare.addEventListener("click", compareFile);
  section.prepend(title, node("p", "Metadata includes project labels, keys, paths and stable hashes. It excludes source content; fingerprints do not establish approval or import readiness."), previewButton, previewStatus, previewError, metadata);
  section.append(disclosure, download, node("p", "Local JSON download only; no project file, receipt or server publication. A selected file plus its 11-byte wrapper must fit the 1 MiB comparison request."), comparisonForm, compare, comparisonStatus, comparisonError, comparison);
  section.setAttribute("aria-busy", "false"); bundlePanels.set(section, invalidatePanel); return section;
}

function exportForm() {
  const form=node("form");form.append(node("h2","Export a redacted static report"));
  const kind=field(form,"Report","select",[["applicability-gap","Applicability counts"],["mapping-collection","Mapping participation"],["trace","Trace summary"]]);
  const target=field(form,"Report destination within project");
  form.append(button("Prepare export",()=>effect("/exports","POST",{report_kind:kind.value,target_path:target.value,format:"static-html",redaction_profile:"strict-default"})));
  form.addEventListener("submit",event=>event.preventDefault());return form;
}
window.addEventListener("beforeunload",event=>{if(dirty){event.preventDefault();event.returnValue="";}});

async function initializeForm(kind) {
  const form=node("form");form.append(node("h2",`Initialize ${kind} review`),node("p","Choose registered Catalogs explicitly. This prepares a new manifest with current hashes and inventories; no scope decisions or relationships are inferred."));
  if(readOnly){form.append(node("p","A writable session is required to initialize decisions."));return form;}
  const resources=(await collection("/resources")).filter(r=>r.role==="oscal-catalog-artifact"&&r.validation_state==="valid");
  if(!resources.length){form.append(node("p","Register a valid Catalog first."));return form;}
  const choices=[["","Choose a Catalog"],...resources.map(r=>[r.resource_id,`${r.key} · ${r.path}`])];
  const framework=field(form,"Framework Catalog","select",choices);
  const destination=field(form,"New decision manifest path within project");
  if(kind==="applicability") {
    form.append(button("Preview initial scope manifest",async()=>{if(form.reportValidity())await effect("/applicability/initializations","POST",{framework_resource_id:framework.value,target_path:destination.value});}));
  } else {
    const source=field(form,"Policy Catalog","select",choices);
    const scope=field(form,"Mapping review scope","select",[["","Choose a scope"],["control-only","Controls only"],["control-plus-statement","Controls and statements"]]);
    const collectionKey=field(form,"Stable collection key");const title=field(form,"Mapping collection title");const version=field(form,"Document version");const time=field(form,"Review time (RFC3339, including timezone)");
    const reviewerKey=field(form,"Reviewer key");const reviewerName=field(form,"Asserted reviewer name");const rationale=field(form,"Intended use and limitations");
    const matching=field(form,"Review matching rationale","select",[["","Choose a rationale"],["semantic","Semantic"],["syntactic","Syntactic"],["functional","Functional"]]);
    const sourceSubject=field(form,"Reviewed policy subject","select",[["","Load subjects after choosing Catalogs"]]);
    const targetSubject=field(form,"Reviewed framework subject","select",[["","Load subjects after choosing Catalogs"]]);
    const reload=async()=>{
      for(const [select,resource,side] of [[sourceSubject,source,"policy"],[targetSubject,framework,"framework"]]) {
        select.replaceChildren();const empty=node("option","Choose a reviewed subject");empty.value="";select.append(empty);
        if(!resource.value)continue;
        const rows=await collection(`/mapping/subjects?resource_id=${encodeURIComponent(resource.value)}&side=${side}`);
        for(const row of rows){if(scope.value==="control-only"&&row.statement_count)continue;const option=node("option",`${row.statement_count?"Statement":"Control"}: ${row.label}`);option.value=JSON.stringify({type:row.statement_count?"statement":"control",id_ref:row.label});select.append(option);}
      }
    };
    form.append(button("Load selected Catalog subjects",reload));
    for(const select of [source,framework,scope])select.addEventListener("change",()=>{sourceSubject.replaceChildren();targetSubject.replaceChildren();});
    const mapKey=field(form,"Stable relationship key");
    const relationship=field(form,"Reviewed relationship","select",[["","Choose a relationship"],...["equivalent-to","equal-to","subset-of","superset-of","intersects-with","no-relationship"].map(value=>[value,value.replaceAll("-"," ")])]);
    const mapRationale=field(form,"Relationship rationale");
    form.append(button("Preview initial mapping manifest",async()=>{if(!form.reportValidity())return;await effect("/mapping/initializations","POST",{source_resource_id:source.value,target_resource_id:framework.value,target_path:destination.value,scope:scope.value,maps:[{key:mapKey.value,relationship:relationship.value,sources:[JSON.parse(sourceSubject.value)],targets:[JSON.parse(targetSubject.value)],reviewer_key:reviewerKey.value,reviewed_at:time.value,rationale:mapRationale.value}],review:{collection:{key:collectionKey.value,title:title.value,version:version.value,last_modified:time.value},reviewers:[{key:reviewerKey.value,type:"person",name:reviewerName.value}],provenance:{method:"human",matching_rationale:matching.value,status:"draft",mapping_description:rationale.value,reviewer_keys:[reviewerKey.value],reviewed_at:time.value}}});}));
  }
  form.append(node("p","After committing, register the manifest as an applicability manifest or mapping collection. Review metadata remains an assertion, not authenticated identity."));
  form.addEventListener("submit",event=>event.preventDefault());return form;
}


/** Retain metadata effects on the consumed additive2.2 and2.3 contracts. */
function bundleEffectsSupported() { return apiMajor === 2 && ["2.2.0", "2.3.0"].includes(apiContractVersion); }

/** Choose report/metadata/source private media family from its preparation route, never its filename. */
function exportDownloadContext(path, id) {
  return path === "/exports" ? id : path === "/project/bundle-exports" ? {family:"metadata-bundle", operation_id:id} : path === "/project/source-bundle-exports" ? {family:"source-bundle", operation_id:id} : undefined;
}

/** Check complete closed index metadata without treating it as source admission or approval. */
function checkedReplacementIndex(value) {
  if (!bundleClosedObject(value,["schema_version","label","resources"]) || !["forge.workspace/1","forge.workspace/2"].includes(value.schema_version) ||
      typeof value.label !== "string" || [...value.label].length < 1 || [...value.label].length > 200 || !Array.isArray(value.resources) || value.resources.length > 1000)
    throw new Error("The replacement returned an unsupported complete index.");
  const keys = new Set(); const paths = new Set(); const roles = workspaceRoles(value.schema_version);
  for (const row of value.resources) {
    if (!bundleClosedObject(row,["key","role","path"]) || typeof row.key !== "string" || !/^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/.test(row.key) ||
        keys.has(row.key) || !roles.includes(row.role) || typeof row.path !== "string" || row.path.length > 512 ||
        !/^[A-Za-z0-9][A-Za-z0-9._-]*(\/[A-Za-z0-9][A-Za-z0-9._-]*)*$/.test(row.path) || paths.has(row.path))
      throw new Error("The replacement returned unsupported or duplicate registration metadata.");
    keys.add(row.key); paths.add(row.path);
  }
  return value;
}

/** Reconcile closed public validation diagnostics before retaining a supposedly valid replacement. */
function checkedImportValidation(value) {
  if (!bundleClosedObject(value,["state","error_count","warning_count","diagnostics"]) || value.state !== "valid" || value.error_count !== 0 ||
      !Number.isSafeInteger(value.warning_count) || value.warning_count < 0 || value.warning_count > 500 || !Array.isArray(value.diagnostics) || value.diagnostics.length > 500) return false;
  let warnings=0;
  for (const item of value.diagnostics) {
    if (!item || typeof item !== "object" || Array.isArray(item) || Object.keys(item).some(key=>!["code","severity","message","resource_id","field"].includes(key)) ||
        !["code","severity","message"].every(key=>Object.hasOwn(item,key)) || typeof item.code !== "string" || [...item.code].length < 3 || [...item.code].length > 100 ||
        !["warning","info"].includes(item.severity) || typeof item.message !== "string" || [...item.message].length < 1 || [...item.message].length > 1000 ||
        item.resource_id !== undefined && item.resource_id !== null && (typeof item.resource_id !== "string" || !/^res_[0-9a-z]{12,80}$/.test(item.resource_id)) ||
        item.field !== undefined && item.field !== null && (typeof item.field !== "string" || [...item.field].length > 256)) return false;
    if (item.severity === "warning") warnings++;
  }
  return warnings === value.warning_count;
}

/** Reconcile the new wrapped preview and complete removed membership before exact receipt review. */
async function checkedBundleImportPreview(value, requestedIndexSchema) {
  if (!bundleClosedObject(value,["validation","preview","replacement"]) || !checkedImportValidation(value.validation) ||
      !value.preview?.preview_id || value.preview.operation_type !== "workspace-index-update" || value.preview.target?.path !== "forge.workspace.json")
    throw new Error("The import did not return a valid exact index-replacement preview.");
  const replacement = value.replacement;
  if (!bundleClosedObject(replacement,["previous_index","proposed_index","supplied_index_sha256","proposed_index_sha256","removed_resource_keys","consumed_file_count"]) ||
      typeof replacement.supplied_index_sha256 !== "string" || !/^[a-f0-9]{64}$/.test(replacement.supplied_index_sha256) ||
      typeof replacement.proposed_index_sha256 !== "string" || !/^[a-f0-9]{64}$/.test(replacement.proposed_index_sha256) ||
      replacement.proposed_index_sha256 !== value.preview.exact_bytes_sha256 || !Number.isSafeInteger(replacement.consumed_file_count) ||
      replacement.consumed_file_count < 0 || replacement.consumed_file_count > 100 || !Array.isArray(replacement.removed_resource_keys) || replacement.removed_resource_keys.length > 100)
    throw new Error("The import returned unsupported replacement hashes or capacity facts.");
  const previous = replacement.previous_index === null ? null : checkedReplacementIndex(replacement.previous_index);
  const proposed = checkedReplacementIndex(replacement.proposed_index);
  if (requestedIndexSchema !== undefined && proposed.schema_version !== `forge.workspace/${requestedIndexSchema}`) throw new Error("The replacement target index version differs from the chosen request.");
  if (previous?.schema_version === "forge.workspace/2" && proposed.schema_version === "forge.workspace/1") throw new Error("Index replacement cannot downgrade index2.");
  const proposedKeys = new Set(proposed.resources.map(row=>row.key));
  const removed = (previous?.resources ?? []).filter(row=>!proposedKeys.has(row.key)).map(row=>row.key);
  if (JSON.stringify(removed) !== JSON.stringify(replacement.removed_resource_keys)) throw new Error("The replacement did not retain complete ordered removed keys.");
  const consumedPaths = new Set([...(previous?.resources ?? []),...proposed.resources].map(row=>row.path));
  if (previous !== null) consumedPaths.add("forge.workspace.json");
  if (replacement.consumed_file_count !== consumedPaths.size)
    throw new Error("The replacement consumed-file facts do not reconcile with complete membership.");
  const normalized = {schema_version:proposed.schema_version,label:proposed.label,resources:proposed.resources.map(row=>({key:row.key,role:row.role,path:row.path}))};
  const digest=new Uint8Array(await crypto.subtle.digest("SHA-256",await new Blob([JSON.stringify(normalized,null,2)+"\n"]).arrayBuffer()));
  const hash=Array.from(digest,byte=>byte.toString(16).padStart(2,"0")).join("");
  if(hash!==replacement.proposed_index_sha256)throw new Error("The complete proposed membership differs from its normalized index hash.");
  return value;
}

/** Show every previous/proposed registration and distinct normalized hashes before confirming replacement. */
function appendBundleReplacement(content, replacement) {
  content.append(node("h3","Complete project index replacement"),node("p","This changes the index label and ordered registrations. Source files are not copied or deleted; local acknowledgment is not domain approval."),
    node("p",`Supplied normalized index SHA-256: ${replacement.supplied_index_sha256}`),node("p",`Proposed normalized index SHA-256: ${replacement.proposed_index_sha256}`),
    node("p",`Consumed physical files: ${replacement.consumed_file_count}. Removed registration keys: ${replacement.removed_resource_keys.join(", ") || "none"}.`));
  for (const [label,index] of [["Previous index",replacement.previous_index],["Proposed index",replacement.proposed_index]]) {
    content.append(node("h4",label));
    if (index === null) content.append(node("p","No project index was present."));
    else content.append(node("p",`${index.schema_version}: ${index.label} (${index.resources.length} registrations)`),table(`${label} complete membership`,[["Key","key"],["Role","role"],["Project-relative path","path"]],index.resources));
  }
}

/** Wrap one chosen file in the 80-byte numeric-selector envelope without parsing or rewriting raw bytes. */
async function bundleImportBody(file, target) {
  if (![1,2].includes(target) || !file || !Number.isSafeInteger(file.size) || file.size < 0 || typeof file.arrayBuffer !== "function") throw new Error("Choose one bundle file and an explicit target index version.");
  const prefix = '{"bundle":'; const suffix = `,"target_index_schema_version":${target},"acknowledge_index_replacement":true}`;
  const overhead = new Blob([prefix,suffix]).size;
  if (file.size > 1024*1024-overhead) throw new Error("Chosen bundle and its 80-byte import wrapper must fit 1MiB.");
  let bytes;
  try { bytes = await file.arrayBuffer(); } catch { throw new Error("The chosen bundle could not be read. Choose it again."); }
  if (!bytes || bytes.byteLength !== file.size) throw new Error("The chosen bundle changed while being read. Choose it again.");
  const body = new Blob([prefix,new Uint8Array(bytes),suffix],{type:"application/json"});
  if (body.size !== file.size+overhead || body.size > 1024*1024) throw new Error("The complete import request exceeds 1MiB.");
  return body;
}

/** Install separate metadata-write forms without granting old query panels new mutation authority. */
function bundleEffectsPanel() {
  const section=node("section");section.setAttribute("data-bundle-panel","");section.setAttribute("data-bundle-effects","");
  section.append(node("h2","Export metadata or replace the index"),node("p","API2 2.2.0 or 2.3.0 metadata-only writes require an exact server receipt. Resource contents, domain approval and multi-file restore are excluded."));
  const error=node("div");error.setAttribute("role","alert");error.tabIndex=-1;error.hidden=true;
  const status=node("p");status.setAttribute("role","status");status.setAttribute("aria-live","polite");
  const exportFields=node("div");const target=fieldInput(exportFields,"Metadata export project-relative target");
  const sensitive=fieldInput(exportFields,"I acknowledge that exported labels, keys, paths and hashes are sensitive metadata.","checkbox");sensitive.required=false;
  const importFields=node("div");const file=fieldInput(importFields,"Choose a bundle for index replacement","file");file.required=false;file.accept=".json,application/json";
  const schema=fieldInput(importFields,"Replacement target index schema","select",[["1","Index 1 (seven roles)"],["2","Index 2 (fifteen roles)"]]);
  const replacement=fieldInput(importFields,"I acknowledge replacing the index label and all registrations; source files will not be deleted.","checkbox");replacement.required=false;
  let generation=0;let busy=false;
  /** Permit only this installed form, writable session and current view epoch to dispatch a preparation. */
  function installed() { return bundleEffectsSupported() && !stopped && activeView === "Trace & Reports" && section.isConnected && element("view").contains(section) && !element("view").inert; }
  /** Preserve focused controls and guard duplicate activation while editing or preparing. */
  function actions() {
    exportButton.setAttribute("aria-disabled",String(!installed() || readOnly || busy || !target.value || !sensitive.checked));
    importButton.setAttribute("aria-disabled",String(!installed() || readOnly || busy || !file.files?.length || !replacement.checked || !["1","2"].includes(schema.value)));
    section.setAttribute("aria-busy",String(busy));
  }
  /** Retire unsent local work and require a new acknowledgment after form values change. */
  function changed(event) { generation++;busy=false;error.hidden=true;dirty=true;if(event.target===target)sensitive.checked=false;if(event.target===file || event.target===schema)replacement.checked=false;actions(); }
  /** Retire stale form callbacks without altering the old query panel or newer focus. */
  function invalidate() { generation++;busy=false;sensitive.checked=false;replacement.checked=false;actions(); }
  /** Keep current local read failures focused without returning the dirty-cancellation sentinel. */
  function localFailure(failure,invoker) { error.textContent=failure instanceof Error?failure.message:"The bundle preparation could not be started.";error.hidden=false;status.textContent="No preparation was confirmed. Review the error and retry explicitly.";if(document.activeElement===invoker && !document.querySelector("dialog[open]"))error.focus(); }
  /** Prepare an explicit metadata destination; existing operation rows own status and receipt review. */
  async function prepareExport() {
    if (!installed() || readOnly || busy || !target.value || !sensitive.checked) return;
    const sequence=++generation;const epoch=pending;const body={target_path:target.value,acknowledge_sensitive_metadata:true};busy=true;error.hidden=true;actions();
    /** Editing, leaving this view or stopping retires an export that has not been dispatched. */
    const current=()=>installed() && sequence===generation && epoch===pending;
    try { await effect("/project/bundle-exports","POST",body,undefined,current); }
    catch(failure) {if(installed() && sequence===generation && epoch===pending)localFailure(failure,exportButton);}
    finally {if(sequence===generation){busy=false;actions();}}
  }
  /** Read the exact chosen file and retain its immutable raw body/key through ordinary request recovery. */
  async function prepareImport() {
    if (!installed() || readOnly || busy || !file.files?.length || !replacement.checked || !["1","2"].includes(schema.value)) return;
    const selected=file.files[0];const targetVersion=Number(schema.value);const sequence=++generation;const epoch=pending;busy=true;error.hidden=true;actions();
    /** A changed selection, navigation or shutdown cannot dispatch the old opaque bytes after pacing. */
    const current=()=>installed() && sequence===generation && epoch===pending;
    try {const body=await bundleImportBody(selected,targetVersion);if(!current())return;await effect("/project/bundle-imports","POST",undefined,body,current,targetVersion);}
    catch(failure) {if(current())localFailure(failure,importButton);}
    finally {if(sequence===generation){busy=false;actions();}}
  }
  const exportButton=node("button","Prepare metadata export");exportButton.type="button";exportButton.addEventListener("click",prepareExport);
  const importButton=node("button","Prepare index replacement");importButton.type="button";importButton.addEventListener("click",prepareImport);
  target.addEventListener("input",changed);sensitive.addEventListener("change",changed);file.addEventListener("change",changed);schema.addEventListener("change",changed);replacement.addEventListener("change",changed);
  section.append(exportFields,exportButton,importFields,importButton,node("p",readOnly?"Read-only session: metadata inspection remains available, but preparing or confirming writes is disabled.":"A chosen file and its 80-byte numeric import wrapper must fit 1MiB. Every incoming file must already exist inside this project; no source is restored."),status,error);
  bundlePanels.set(section,invalidate);actions();return section;
}

/** Require the consumed source contract; earlier numeric API2 sessions retain their existing views. */
function sourceBundleEffectsSupported() { return apiMajor === 2 && apiContractVersion === "2.3.0"; }

/** Validate portable project-relative display facts; native confinement remains server-owned. */
function sourceRestorePath(value) {
  if (typeof value !== "string" || value.length < 1 || value.length > 512 || !/^[A-Za-z0-9][A-Za-z0-9._-]*(\/[A-Za-z0-9][A-Za-z0-9._-]*)*$/.test(value)) return false;
  return value.split("/").every(segment => !segment.endsWith(".") && !/^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$/i.test(segment.split(".",1)[0]));
}

/** Admit only exact bounded counters, never boolean/string counter substitutions. */
function sourceRestoreCount(value, maximum = 100) { return Number.isSafeInteger(value) && value >= 0 && value <= maximum; }

/** Require exact server-issued lookup/preview IDs without making the public ID authorizing. */
function sourceRestoreId(value, prefix = "op") { return typeof value === "string" && new RegExp(`^${prefix}_[0-9a-z]{12,80}$`).test(value); }

/** Check a lowercase exact-byte or normalized-index SHA-256 fact. */
function sourceRestoreHash(value) { return typeof value === "string" && /^[0-9a-f]{64}$/.test(value); }

/** Check a current/proposed file generation and explicitly nullable registration identity. */
function checkedSourceFileFact(value, prior = false) {
  const fields = ["key","role","sha256","size",...(prior ? ["resource_id"] : [])];
  if (!bundleClosedObject(value,fields) || !sourceRestoreHash(value.sha256) || !sourceRestoreCount(value.size,10*1024*1024) ||
      value.key !== null && (typeof value.key !== "string" || !/^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/.test(value.key)) ||
      value.role !== null && !workspaceRoles("forge.workspace/2").includes(value.role) || (value.key === null) !== (value.role === null) ||
      prior && ((value.key === null) !== (value.resource_id === null) || value.resource_id !== null && !sourceRestoreId(value.resource_id,"res")))
    throw new Error("The source restore returned unsupported file-generation metadata.");
  return value;
}

/** Check every closed batch target, binding and directory before retaining a confirmation receipt. */
function checkedSourceRestorePreview(value) {
  if (!bundleClosedObject(value,["preview_id","operation_id","operation_type","snapshot_version","observed_batch_version","exact_manifest_sha256","targets","input_bindings","directories","validation","semantic_summary","receipt"]) ||
      !sourceRestoreId(value.preview_id,"prev") || !sourceRestoreId(value.operation_id) || value.operation_type !== "project-source-restore" ||
      ![value.snapshot_version,value.observed_batch_version,value.exact_manifest_sha256].every(sourceRestoreHash) || !checkedImportValidation(value.validation) ||
      typeof value.semantic_summary !== "string" || [...value.semantic_summary].length > 4000 ||
      !bundleClosedObject(value.receipt,["token","expires_at"]) || typeof value.receipt.token !== "string" || value.receipt.token.length < 16 || value.receipt.token.length > 512 || typeof value.receipt.expires_at !== "string" || !Number.isFinite(Date.parse(value.receipt.expires_at)) ||
      !Array.isArray(value.targets) || !value.targets.length || value.targets.length > 100 || !Array.isArray(value.input_bindings) || value.input_bindings.length > 100 || !Array.isArray(value.directories) || value.directories.length > 100)
    throw new Error("The source restore did not return a complete valid batch preview.");
  const targets=new Map();let diffBytes=0;
  for (const target of value.targets) {
    if (!bundleClosedObject(target,["path","kind","key","role","status","base_sha256","base_size","target_version","exact_bytes_sha256","size","diff_text","diff_truncated","binary"]) ||
        !sourceRestorePath(target.path) || targets.has(target.path.toLowerCase()) || !["index","resource"].includes(target.kind) || !["create","overwrite"].includes(target.status) ||
        typeof target.target_version !== "string" || target.target_version.length < 8 || target.target_version.length > 128 || typeof target.diff_text !== "string" ||
        typeof target.diff_truncated !== "boolean" || typeof target.binary !== "boolean" ||
        (target.path === "forge.workspace.json") !== (target.kind === "index") || (target.kind === "index") !== (target.key === null))
      throw new Error("The source restore returned unsupported or duplicate complete targets.");
    checkedSourceFileFact({key:target.key,role:target.role,sha256:target.exact_bytes_sha256,size:target.size});
    if (target.status === "overwrite" ? !sourceRestoreHash(target.base_sha256) || !sourceRestoreCount(target.base_size,10*1024*1024) : target.base_sha256 !== null || target.base_size !== null)
      throw new Error("The source restore did not identify each observed target base.");
    diffBytes += new Blob([target.diff_text]).size; targets.set(target.path.toLowerCase(),target);
  }
  if (value.targets.at(-1).path !== "forge.workspace.json" || diffBytes > 200000) throw new Error("The source restore order or shared diff display bound is unsupported.");
  const bindings=new Map();
  for (const binding of value.input_bindings) {
    if (!bundleClosedObject(binding,["path","kind","current","proposed"]) || !sourceRestorePath(binding.path) || bindings.has(binding.path.toLowerCase()) ||
        !["index","resource"].includes(binding.kind) || (binding.path === "forge.workspace.json") !== (binding.kind === "index") || binding.current === null && binding.proposed === null)
      throw new Error("The source restore returned unsupported complete input bindings.");
    if (binding.current !== null) checkedSourceFileFact(binding.current,true);
    if (binding.proposed !== null) checkedSourceFileFact(binding.proposed);
    bindings.set(binding.path.toLowerCase(),binding);
  }
  for (const target of value.targets) {
    const binding=bindings.get(target.path.toLowerCase());
    if (!binding || binding.path !== target.path || !binding.proposed || ["key","role","size"].some(key=>binding.proposed[key] !== target[key]) || binding.proposed.sha256 !== target.exact_bytes_sha256 ||
        (target.status === "create" ? binding.current !== null : !binding.current || binding.current.sha256 !== target.base_sha256 || binding.current.size !== target.base_size))
      throw new Error("The source restore targets and complete observed/proposed bindings disagree.");
  }
  const directories=[];
  for (const directory of value.directories) {
    if (!bundleClosedObject(directory,["path","status","nearest_existing_parent_version"]) || !sourceRestorePath(directory.path) || directory.status !== "create" ||
        directories.some(path=>path.toLowerCase()===directory.path.toLowerCase()) || typeof directory.nearest_existing_parent_version !== "string" ||
        directory.nearest_existing_parent_version.length < 8 || directory.nearest_existing_parent_version.length > 128)
      throw new Error("The source restore returned unsupported directory intentions.");
    directories.push(directory.path);
  }
  if (directories.some((path,position)=>directories.slice(position+1).some(parent=>path.startsWith(parent+"/")))) throw new Error("The source restore directory plan is not parent-first.");
  return value;
}

/** Validate full explicit index membership and portable aliases without granting domain approval. */
function checkedSourceReplacementIndex(value) {
  checkedReplacementIndex(value);
  if (/[\x00-\x1f\x7f-\x9f]/.test(value.label)) throw new Error("The source replacement label contains unsupported control characters.");
  const paths=new Set();
  for (const row of value.resources) {
    const path=row.path.toLowerCase();
    if (!sourceRestorePath(row.path) || path === "forge.workspace.json" || paths.has(path)) throw new Error("The source replacement returned nonportable or aliased paths.");
    paths.add(path);
  }
  return value;
}

/** Hash admitted ordered index fields plus LF, distinct from original uploaded bundle bytes. */
async function sourceReplacementHash(index) {
  const ordered={schema_version:index.schema_version,label:index.label,resources:index.resources.map(row=>({key:row.key,role:row.role,path:row.path}))};
  const blob=new Blob([JSON.stringify(ordered,null,2)+"\n"]);
  if (blob.size > 1024*1024) throw new Error("The complete proposed index exceeds its supported byte bound.");
  return Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256",await blob.arrayBuffer())),byte=>byte.toString(16).padStart(2,"0")).join("");
}

/** Reconcile every index/target/removal relation before a source receipt enters the UI. */
async function checkedSourceImportPreview(value, requestedIndexSchema) {
  if (!bundleClosedObject(value,["validation","preview","replacement"]) || !checkedImportValidation(value.validation)) throw new Error("The source import did not return a valid complete preview.");
  const preview=checkedSourceRestorePreview(value.preview); const replacement=value.replacement;
  if (!sourceRestoreEqual(value.validation,preview.validation) || !bundleClosedObject(replacement,["previous_index","proposed_index","supplied_index_sha256","proposed_index_sha256","removed_resource_keys","consumed_file_count"]) ||
      !sourceRestoreHash(replacement.supplied_index_sha256) || !sourceRestoreHash(replacement.proposed_index_sha256) || !sourceRestoreCount(replacement.consumed_file_count) ||
      replacement.consumed_file_count !== preview.input_bindings.length || !Array.isArray(replacement.removed_resource_keys)) throw new Error("The source import replacement facts do not reconcile.");
  const previous=replacement.previous_index === null ? null : checkedSourceReplacementIndex(replacement.previous_index);
  const proposed=checkedSourceReplacementIndex(replacement.proposed_index);
  if (![1,2].includes(requestedIndexSchema) || proposed.schema_version !== `forge.workspace/${requestedIndexSchema}` || proposed.resources.length > 99 ||
      previous?.schema_version === "forge.workspace/2" && proposed.schema_version === "forge.workspace/1") throw new Error("The source replacement does not match the explicit index version or would downgrade it.");
  const oldRows=previous?.resources ?? []; const incomingKeys=new Set(proposed.resources.map(row=>row.key));
  const removed=oldRows.filter(row=>!incomingKeys.has(row.key)).map(row=>row.key);
  if (JSON.stringify(removed) !== JSON.stringify(replacement.removed_resource_keys)) throw new Error("The source replacement omitted or reordered removed registration keys.");
  const expected=new Set(["forge.workspace.json",...oldRows.map(row=>row.path),...proposed.resources.map(row=>row.path)]);
  if (expected.size !== preview.input_bindings.length || preview.input_bindings.some(row=>!expected.has(row.path))) throw new Error("The complete planned path union differs from the replacement membership.");
  const oldByPath=new Map(oldRows.map(row=>[row.path,row])); const incomingPaths=new Set(["forge.workspace.json",...proposed.resources.map(row=>row.path)]);
  for (const binding of preview.input_bindings) {
    const old=oldByPath.get(binding.path);
    if (old ? !binding.current || binding.current.key !== old.key || binding.current.role !== old.role : binding.current && (binding.current.key !== null || binding.current.role !== null || binding.current.resource_id !== null))
      throw new Error("The source restore mislabeled a registered or unregistered observed base.");
    if ((binding.proposed !== null) !== incomingPaths.has(binding.path)) throw new Error("The source restore proposed bindings differ from complete incoming membership.");
  }
  const expectedTargets=[...proposed.resources.map(row=>[row.path,"resource",row.key,row.role]),["forge.workspace.json","index",null,null]];
  if (JSON.stringify(preview.targets.map(row=>[row.path,row.kind,row.key,row.role])) !== JSON.stringify(expectedTargets) ||
      preview.targets.at(-1).exact_bytes_sha256 !== replacement.proposed_index_sha256 || (previous === null) !== (preview.input_bindings.find(row=>row.path==="forge.workspace.json").current === null) ||
      await sourceReplacementHash(proposed) !== replacement.proposed_index_sha256) throw new Error("The complete index target, normalized hash or original absence differs from the replacement.");
  return {family:"source-restore",preview,replacement};
}

/** Retain exact opaque selected bytes in the explicit 147-byte source-import envelope. */
async function sourceBundleImportBody(file, target) {
  if (![1,2].includes(target) || !file || !sourceRestoreCount(file.size,1048429) || typeof file.arrayBuffer !== "function") throw new Error("Choose a source bundle no larger than 1048429 bytes and an explicit index version.");
  const prefix='{"bundle":'; const suffix=`,"target_index_schema_version":${target},"acknowledge_index_replacement":true,"acknowledge_source_content":true,"acknowledge_replace_files":true}`;
  const overhead=new Blob([prefix,suffix]).size;
  if (overhead !== 147) throw new Error("The source import wrapper is unsupported.");
  let bytes;
  try {bytes=await file.arrayBuffer();} catch {throw new Error("The selected source bundle could not be read. Select it again.");}
  if (!bytes || bytes.byteLength !== file.size) throw new Error("The selected source bundle changed while being read. Select it again.");
  const body=new Blob([prefix,new Uint8Array(bytes),suffix],{type:"application/json"});
  if (body.size !== file.size+147 || body.size > 1024*1024) throw new Error("The complete source import exceeds 1MiB.");
  return body;
}

/** Validate complete committed byte facts separately from cleanup qualification. */
function checkedSourceRestoreResult(value, cleanup) {
  if (!bundleClosedObject(value,["write_committed","exact_manifest_sha256","committed_targets","cleanup_state"]) || value.write_committed !== true || !sourceRestoreHash(value.exact_manifest_sha256) ||
      value.cleanup_state !== cleanup || !["verified","pending","unverified"].includes(cleanup) || !Array.isArray(value.committed_targets) || !value.committed_targets.length || value.committed_targets.length > 100)
    throw new Error("The source restore returned unsupported committed byte facts.");
  const paths=new Set();
  for (const target of value.committed_targets) {
    if (!bundleClosedObject(target,["path","sha256","size"]) || !sourceRestorePath(target.path) || paths.has(target.path.toLowerCase()) || !sourceRestoreHash(target.sha256) || !sourceRestoreCount(target.size,10*1024*1024)) throw new Error("The source restore returned unsupported committed target membership.");
    paths.add(target.path.toLowerCase());
  }
  if (value.committed_targets.at(-1).path !== "forge.workspace.json") throw new Error("The source restore did not report the committed index last.");
  return value;
}

/** Check staging counters and batch outcome/cleanup pairs; cancellation acknowledgment is not rollback. */
function checkedSourceRestoreOperation(value, id) {
  if (!bundleClosedObject(value,["operation_id","kind","state","created_at","updated_at","cancel_requested","write_outcome","progress","result","error","cleanup_state"]) || value.operation_id !== id || !sourceRestoreId(id) ||
      value.kind !== "bundle-restore" || !["pending","running","succeeded","failed","cancelled","recovery-required"].includes(value.state) ||
      !["unmeasured","none","committed","unknown"].includes(value.write_outcome) || !["unmeasured","verified","pending","unverified"].includes(value.cleanup_state) || typeof value.cancel_requested !== "boolean" ||
      typeof value.created_at !== "string" || typeof value.updated_at !== "string" || !Number.isFinite(Date.parse(value.created_at)) || !Number.isFinite(Date.parse(value.updated_at))) throw new Error("The source restore returned an unsupported operation identity or state.");
  const active=["pending","running"].includes(value.state);
  if (value.progress !== null && (!bundleClosedObject(value.progress,["completed_files","total_files"]) || !sourceRestoreCount(value.progress.completed_files) || !sourceRestoreCount(value.progress.total_files) ||
      value.progress.completed_files > value.progress.total_files || !active || value.cancel_requested)) throw new Error("The source restore returned unsupported staging counters.");
  if (active) {
    if (value.write_outcome !== "unmeasured" || value.result !== null || value.error !== null) throw new Error("The source restore acknowledgment invented a terminal write outcome.");
  } else if (value.state === "succeeded") {
    if (value.write_outcome !== "committed" || value.error !== null) throw new Error("The source restore success did not verify a committed outcome.");
    checkedSourceRestoreResult(value.result,value.cleanup_state);
  } else {
    if (["failed","cancelled"].includes(value.state) && (value.write_outcome !== "none" || value.cleanup_state !== "verified" || value.result !== null) || value.state === "recovery-required" && value.write_outcome === "unmeasured")
      throw new Error("The source restore terminal state does not establish its declared rollback or recovery outcome.");
    if (value.result !== null) {
      if (value.state !== "recovery-required" || value.write_outcome !== "committed") throw new Error("The source restore terminal result conflicts with its write outcome.");
      checkedSourceRestoreResult(value.result,value.cleanup_state);
    }
    if (!(value.state === "cancelled" && value.error === null)) {
      const error=value.error;
      if (!error || typeof error !== "object" || Array.isArray(error) || !["code","message","retryable"].every(key=>Object.hasOwn(error,key)) || Object.keys(error).some(key=>!["code","message","retryable","correlation_id","field","resource","resource_version"].includes(key)) ||
          typeof error.code !== "string" || !error.code.length || error.code.length > 100 || typeof error.message !== "string" || !error.message.length || [...error.message].length > 500 || typeof error.retryable !== "boolean") throw new Error("The source restore terminal error was not a supported safe error.");
    }
  }
  return value;
}

/** Keep lookup rows connected across navigation; stopped pages retain IDs without reviving credentials. */
function sourceRestoreCurrent(row) { return sourceBundleEffectsSupported() && !stopped && row.article.isConnected && sourceRestoreRows.get(row.id) === row; }

/** Keep focused native controls connected while preventing duplicate reads/cancellations. */
function sourceRestoreActions(row) {
  row.check.setAttribute("aria-disabled",String(stopped || !sourceBundleEffectsSupported() || row.checking || row.cancelling || row.polling));
  row.cancel.setAttribute("aria-disabled",String(readOnly || stopped || !sourceBundleEffectsSupported() || row.cancelling || row.terminal || row.cancelRequested || row.unknown || !row.last));
}

/** Preserve the known public lookup after loss;404/expiry never proves no write or authorizes resend. */
function sourceRestoreUnknown(row, failure) {
  if(!row.terminal) {row.unknown=true;
  row.status.textContent="Source restore outcome is unverified. Keep this operation ID and check it explicitly in a fresh same-project API2 2.3.0 session. A missing outcome is not proof that no files changed; do not resend confirmation.";}
  row.error.textContent=failure instanceof Error ? failure.message : "The source restore outcome could not be verified.";row.error.hidden=false;
  sourceRestoreActions(row);
}

/** Preserve validated outcome facts and transfer only focus owned by replaced result content. */
function acceptSourceRestore(row, value) {
  const checked=checkedSourceRestoreOperation(value,row.id); const active=["pending","running"].includes(checked.state);
  if (row.terminal && active) return;
  if (row.last?.write_outcome === "committed" && checked.write_outcome !== "committed") throw new Error("The source restore returned a conflicting committed outcome. Retain the ID and inspect recovery.");
  if(row.last && Date.parse(checked.updated_at)<Date.parse(row.last.updated_at))return;
  if(row.last && !["pending","running","recovery-required"].includes(row.last.state) && checked.state!==row.last.state && !(row.last.state==="succeeded" && checked.state==="recovery-required" && checked.write_outcome==="committed"))return;
  if (row.progress && checked.progress && (row.progress.total_files !== checked.progress.total_files || row.progress.completed_files > checked.progress.completed_files)) throw new Error("The source restore returned inconsistent staging counters.");
  if (row.expected && checked.result && (checked.result.exact_manifest_sha256 !== row.expected.exact_manifest_sha256 ||
      !sourceRestoreEqual(checked.result.committed_targets,row.expected.targets.map(target=>({path:target.path,sha256:target.exact_bytes_sha256,size:target.size}))))) throw new Error("The source restore result does not match the complete confirmed byte plan.");
  const previousResult=row.last?.result;
  row.commitAttempted=true;row.last=checked;row.cancelRequested ||= checked.cancel_requested;row.terminal=!active;row.unknown=false;row.error.hidden=true;if(checked.progress)row.progress=checked.progress;
  const progress=checked.progress ? `Staged files: ${checked.progress.completed_files} of ${checked.progress.total_files} reported.` : "Progress outside measured staging is indeterminate.";
  const cleanup=checked.cleanup_state === "verified" ? "Cleanup verified." : `Cleanup ${checked.cleanup_state}; project access may remain blocked until qualified recovery.`;
  row.status.textContent=active ? `Source restore ${checked.state}. ${row.cancelRequested ? "Cancellation requested; awaiting terminal outcome. " : ""}${progress}` :
    checked.state === "succeeded" ? `Exact source bytes committed. ${cleanup} Refresh the project view explicitly.` :
    ["failed","cancelled"].includes(checked.state) ? `Source restore ${checked.state}; verified rollback reports no committed write. ${cleanup}` :
    `Source restore requires recovery. Write outcome: ${checked.write_outcome}. ${cleanup} Keep this operation ID; do not resend confirmation.`;
  if (checked.error) {row.error.textContent=`${checked.error.code}: ${checked.error.message}`;row.error.hidden=false;}
  if(!sourceRestoreEqual(previousResult ?? null,checked.result)) {
    const transferFocus=row.result.contains(document.activeElement);row.result.replaceChildren();
    if(checked.result)row.result.append(node("p",`Committed manifest SHA-256: ${checked.result.exact_manifest_sha256}`),table("Complete committed source targets",[["Path","path"],["Exact SHA-256","sha256"],["Bytes","size"]],checked.result.committed_targets));
    if(transferFocus && sourceRestoreCurrent(row))row.status.focus();
  }
  sourceRestoreActions(row);
}

/** Create one bounded persistent nonauthorizing lookup row before a confirmation can leave the page. */
function sourceRestoreRow(id, expected) {
  if (!sourceRestoreId(id)) throw new Error("The source restore lookup ID is unsupported.");
  const expectedFacts=expected ? {exact_manifest_sha256:expected.exact_manifest_sha256,targets:expected.targets.map(target=>({path:target.path,exact_bytes_sha256:target.exact_bytes_sha256,size:target.size}))} : undefined;
  const existing=sourceRestoreRows.get(id);
  if(existing) {
    if(expectedFacts && existing.expected && !sourceRestoreEqual(expectedFacts,existing.expected))throw new Error("The source restore reused a lookup ID for a different complete plan.");
    if(expectedFacts)existing.expected=expectedFacts;return existing;
  }
  if (sourceRestoreRows.size >= 256) throw new Error("The supported source outcome display bound was reached. Retain existing IDs and open a fresh same-project session.");
  const article=node("article");article.setAttribute("data-source-restore-row",id);
  const heading=node("h2",`Source restore ${id}`);heading.id=`source-restore-title-${id}`;article.setAttribute("aria-labelledby",heading.id);const status=node("p",expected ? "Preparation is ready for review. No restore has been confirmed; this lookup may return not-found until an intent is accepted." : "This supplied operation ID has not yet been verified and does not authorize a restore. Check its outcome explicitly.");
  status.setAttribute("role","status");status.setAttribute("aria-live","polite");status.setAttribute("data-source-restore-status","");status.setAttribute("aria-label",`Source restore ${id} status`);status.tabIndex=-1;
  const error=node("div");error.setAttribute("role","alert");error.setAttribute("data-source-restore-error","");error.tabIndex=-1;error.hidden=true;
  const result=node("div");
  const row={id,article,status,error,result,expected:expectedFacts,last:null,generation:0,progress:null,polling:false,checking:false,cancelling:false,terminal:false,cancelRequested:false,unknown:false,commitAttempted:false};
  row.check=node("button","Check source restore status");row.check.type="button";
  row.check.addEventListener("click",async()=>{
    if(!sourceRestoreCurrent(row) || row.check.getAttribute("aria-disabled")==="true")return;
    const focused=document.activeElement===row.check;
    await pollSourceRestore(row,true);
    if(focused && sourceRestoreCurrent(row) && document.activeElement===row.check)row.status.focus();
  });
  row.cancel=node("button","Request source restore cancellation");row.cancel.type="button";
  row.cancel.addEventListener("click",async()=>{
    if(!sourceRestoreCurrent(row) || row.cancel.getAttribute("aria-disabled")==="true")return;
    row.cancelling=true;const generation=++row.generation;sourceRestoreActions(row);
    try {const value=await api(`/project/bundle-restores/${encodeURIComponent(id)}/cancel`,"POST",{});if(!sourceRestoreCurrent(row) || generation!==row.generation)return;
      const checked=checkedSourceRestoreOperation(value,id);if(checked.cancel_requested!==true && ["pending","running"].includes(checked.state))throw new Error("The source restore cancellation was not acknowledged.");acceptSourceRestore(row,checked);
    }catch(failure){if(sourceRestoreCurrent(row) && generation===row.generation)sourceRestoreUnknown(row,failure);}
    finally{row.cancelling=false;if(sourceRestoreCurrent(row))sourceRestoreActions(row);}
  });
  article.append(heading,status,node("p","Keep this nonauthorizing operation ID for explicit lookup after reconnect. Session receipts are not restored."),error,row.check,row.cancel,result);
  operationRegion().append(article);sourceRestoreRows.set(id,row);sourceRestoreActions(row);return row;
}

/** Poll only GET for a known ID with one65-second caller budget; obsolete errors cannot erase cancellation. */
async function pollSourceRestore(row, once = false) {
  if(!sourceRestoreCurrent(row) || row.polling || row.checking)return;
  row.polling=!once;row.checking=once;sourceRestoreActions(row);const deadline=performance.now()+65000;
  try {
    for(let reads=0;reads<130 && sourceRestoreCurrent(row);reads++) {
      if(!once && (row.terminal || row.unknown))return;
      if(performance.now()>=deadline)throw new Error("Source restore polling reached its caller budget. Check the known ID explicitly; do not resend confirmation.");
      if(reads>0)await new Promise(resolve=>setTimeout(resolve,500));
      if(!sourceRestoreCurrent(row))return;
      if(performance.now()>=deadline)throw new Error("Source restore polling reached its caller budget. Check this ID explicitly.");
      const generation=row.generation;
      let value;
      try{value=await api(`/project/bundle-restores/${encodeURIComponent(row.id)}`,"GET",undefined,undefined,undefined,()=>sourceRestoreCurrent(row) && performance.now()<deadline);}
      catch(failure){if(!sourceRestoreCurrent(row))return;if(generation!==row.generation)continue;throw failure;}
      if(!sourceRestoreCurrent(row))return;if(generation!==row.generation)continue;acceptSourceRestore(row,value);if(once || row.terminal)return;
    }
    if(sourceRestoreCurrent(row) && !row.terminal)sourceRestoreUnknown(row,new Error("Source restore polling reached its read bound. Check this ID explicitly."));
  }catch(failure){if(sourceRestoreCurrent(row))sourceRestoreUnknown(row,failure);}
  finally{row.polling=false;row.checking=false;if(sourceRestoreCurrent(row))sourceRestoreActions(row);}
}

/** Retain public IDs while retiring active checks and every old-session confirmation after shutdown. */
function stopSourceRestoreRows() {
  for(const row of sourceRestoreRows.values()) {row.generation++;row.status.textContent="Workspace stopped. Keep this operation ID and look it up explicitly in a fresh same-project API2 2.3.0 session. Old receipts cannot be reused; not-found does not prove no write.";sourceRestoreActions(row);}
}

/** Render every captured generation using declared text fields, never hidden receipt tokens or source objects. */
function appendSourceReplacement(content, context) {
  const {preview,replacement}=context;
  content.append(node("p",preview.semantic_summary),node("p",`Snapshot: ${preview.snapshot_version}. Observed batch: ${preview.observed_batch_version}. Exact manifest: ${preview.exact_manifest_sha256}.`),
    node("p",`Outcome lookup ID: ${preview.operation_id}. Receipt expires: ${preview.receipt.expires_at}.`),
    node("p",`Complete planned files: ${replacement.consumed_file_count} of100; index included even if absent. Removed registration keys: ${replacement.removed_resource_keys.join(", ") || "none"}. Files removed from membership are retained on disk.`));
  content.append(node("h3","Every target in publication order; index last"));
  for(const target of preview.targets) {
    const section=node("section");section.append(node("h4",`${target.status}: ${target.path}`),node("p",`${target.kind}. Registration key: ${target.key ?? "none"}. Role: ${target.role ?? "none"}.`),
      node("p",`Observed base: ${target.base_sha256 ?? "absent"}; bytes: ${target.base_size ?? "absent"}; version: ${target.target_version}.`),node("p",`Exact proposed SHA-256: ${target.exact_bytes_sha256}; bytes: ${target.size}.`),
      node("p",target.binary ? "Binary bytes are bound by the complete hash; no text diff is available." : target.diff_truncated ? "Diff truncated within the shared display budget; the hash binds all bytes." : "Text diff; the hash binds all bytes."),Object.assign(node("pre",target.diff_text),{tabIndex:0}));content.append(section);
  }
  content.append(node("h3","All current and proposed input bindings"));
  for(const binding of preview.input_bindings) {
    content.append(node("h4",`${binding.kind}: ${binding.path}`));
    for(const [label,fact] of [["Current",binding.current],["Proposed",binding.proposed]]) content.append(node("p",fact ? `${label}: key ${fact.key ?? "unregistered"}; role ${fact.role ?? "none"}; ${label==="Current" ? `resource ID ${fact.resource_id ?? "none"}; ` : ""}SHA-256 ${fact.sha256}; bytes ${fact.size}.` : `${label}: absent.`));
  }
  content.append(table("Complete new directory intentions",[["Directory","path"],["Action","status"],["Nearest existing parent version","nearest_existing_parent_version"]],preview.directories),
    node("h3","Complete index replacement"),node("p",`Supplied normalized index SHA-256: ${replacement.supplied_index_sha256}. Proposed normalized index SHA-256: ${replacement.proposed_index_sha256}.`));
  for(const [label,index] of [["Previous index",replacement.previous_index],["Proposed index",replacement.proposed_index]]) {
    content.append(node("h4",label));if(index===null)content.append(node("p","No index was present."));else content.append(node("p",`${index.schema_version}: ${index.label}; ${index.resources.length} registrations.`),table(`${label} complete source restore membership`,[["Key","key"],["Role","role"],["Path","path"]],index.resources));
  }
  content.append(node("p","Confirmation may create directories and replace every listed file. Workspace reads are fenced during publication/recovery; external CLI or editor readers may observe mixed whole-file generations. This receipt is not domain approval."));
}

/** Own one complete batch dialog; dismissal/new views invalidate the receipt UI, never the persistent lookup. */
async function previewSourceRestore(context, isCurrent) {
  if(!sourceBundleEffectsSupported() || !isCurrent() || stopped)return false;
  const sequence=++previewGeneration;const epoch=pending;
  const checked=await checkedSourceImportPreview({validation:context.preview.validation,preview:context.preview,replacement:context.replacement},Number(context.replacement.proposed_index.schema_version.at(-1)));
  if(!isCurrent() || stopped || sequence!==previewGeneration || epoch!==pending)return false;
  if(previewClosePending)await previewClosePending;
  if(!isCurrent() || stopped || sequence!==previewGeneration || epoch!==pending)return false;
  const row=sourceRestoreRow(checked.preview.operation_id,checked.preview);const dialog=element("preview-dialog");const content=element("preview-content");
  let attempted=row.commitAttempted;let busy=false;let closingConfirmed=false;const key=crypto.randomUUID();
  /** Only this open dialog may consume its receipt or publish its close/error/focus result. */
  function current(requireOpen=true) {return !stopped && sourceBundleEffectsSupported() && sequence===previewGeneration && epoch===pending && dialog.isConnected && (requireOpen ? dialog.open : !document.querySelector("dialog[open]"));}
  /** Native Escape retires immediately; queued closes cannot invalidate a later preview. */
  function invalidate(event) {
    if(event.type==="cancel")previewCloseLifecycle(dialog);
    if(event.type==="close" && dialog.open)return;
    if(event.type==="close"){dialog.removeEventListener("cancel",invalidate);dialog.removeEventListener("close",invalidate);}
    if(sequence===previewGeneration && (!closingConfirmed || event.type==="cancel"))previewGeneration++;
  }
  /** Require complete acknowledgment, keep focused native controls enabled and prevent repeated commit. */
  function actions() {confirm.setAttribute("aria-disabled",String(readOnly || attempted || row.commitAttempted || busy || !ack.checked || !current()));ack.disabled=attempted;}
  /** Display only this still-owned dialog's safe failure, retaining the known ID after uncertain dispatch. */
  function localFailure(failure) {if(!current())return;error.textContent=failure instanceof Error?failure.message:"The restore confirmation could not be verified.";error.hidden=false;error.focus();}
  /** Dismiss immediately retires this UI receipt and waits for the native invoking-element lifecycle. */
  function dismiss() {if(sequence===previewGeneration)previewGeneration++;return previewCloseLifecycle(dialog,true);}
  /** Send one acknowledged batch; accepted/lost replies retain GET-only recovery without clearing newer forms. */
  async function confirmRestore() {
    if(!current() || readOnly || attempted || row.commitAttempted || busy || !ack.checked)return;
    row.commitAttempted=true;attempted=true;busy=true;error.hidden=true;actions();row.generation++;
    row.status.textContent="Confirmation sent or awaiting transport; outcome not yet verified. Keep this operation ID and never automatically resend.";
    const request={receipt:checked.preview.receipt.token,observed_batch_version:checked.preview.observed_batch_version,acknowledge_exact_restore:true};
    try {
      const value=await api(`/project/bundle-restores/${encodeURIComponent(checked.preview.preview_id)}/commit`,"POST",request,key,undefined,()=>current());
      if(!sourceRestoreCurrent(row))return;acceptSourceRestore(row,value);
      if(current()){closingConfirmed=true;await previewCloseLifecycle(dialog,true);if(current(false))row.status.focus();}
      if(sourceRestoreCurrent(row) && !row.terminal)void pollSourceRestore(row);
    }catch(failure){if(sourceRestoreCurrent(row))sourceRestoreUnknown(row,failure);localFailure(new Error("The restore reply was not verified. Keep the displayed operation ID, check its status explicitly, and do not resend confirmation."));}
    finally{busy=false;if(current())actions();}
  }
  dialog.addEventListener("cancel",invalidate);dialog.addEventListener("close",invalidate);dialog.querySelector("[role=alert]")?.remove();
  content.replaceChildren(Object.assign(node("h2","Review complete exact source restore"),{id:"preview-title"}));
  const facts=node("section");facts.setAttribute("data-bundle-panel","");appendSourceReplacement(facts,checked);content.append(facts);
  if(attempted)content.append(node("p","This preview already has a confirmation attempt. Only check the known outcome ID; reopening this dialog cannot resend the receipt."));
  const error=node("div");error.setAttribute("role","alert");error.tabIndex=-1;error.hidden=true;
  const ack=fieldInput(content,"I reviewed every target, input binding, directory and index replacement; restore these exact bytes","checkbox");ack.required=false;
  const confirm=node("button","Confirm this exact source restore");confirm.type="button";confirm.addEventListener("click",confirmRestore);
  ack.addEventListener("change",actions);content.append(error,button("Keep editing",dismiss),confirm);dialog.showModal();content.querySelector("h2").tabIndex=-1;content.querySelector("h2").focus();actions();return true;
}

/** Install opt-in source preparation and explicit fresh-session lookup without changing metadata query authority. */
function sourceBundleEffectsPanel() {
  const section=node("section");section.setAttribute("data-bundle-panel","");section.setAttribute("data-source-bundle-effects","");
  section.append(node("h2","Export or restore exact sources"),node("p","Source bytes, labels, keys, relative paths and hashes may be sensitive. Preparing a bundle does not approve domain data. Restore requires review and explicit confirmation of every target, directory and index."));
  const error=node("div");error.setAttribute("role","alert");error.tabIndex=-1;error.hidden=true;const status=node("p");status.setAttribute("role","status");status.setAttribute("aria-live","polite");
  const exportFields=node("div");const target=fieldInput(exportFields,"Source export project-relative target");
  const exportAck=fieldInput(exportFields,"Include exact source bytes and sensitive metadata in this export","checkbox");exportAck.required=false;
  const importFields=node("div");const file=fieldInput(importFields,"Choose an exact source bundle JSON file","file");file.required=false;file.accept=".json,application/json";
  const schema=fieldInput(importFields,"Source restore target index schema","select",[["1","Index 1 (seven roles)"],["2","Index 2 (fifteen roles)"]]);
  const indexAck=fieldInput(importFields,"I acknowledge replacing the complete project index label and registrations","checkbox");indexAck.required=false;
  const sourceAck=fieldInput(importFields,"I acknowledge including exact source bytes and sensitive metadata","checkbox");sourceAck.required=false;
  const filesAck=fieldInput(importFields,"I understand the complete preview may create or overwrite project files","checkbox");filesAck.required=false;
  const lookupFields=node("div");const lookup=fieldInput(lookupFields,"Source restore outcome operation ID");lookup.required=false;let generation=0;let busy=false;
  /** Installed view ownership fences raw file reads and delayed preparation dispatch. */
  function installed() {return sourceBundleEffectsSupported() && !stopped && activeView==="Trace & Reports" && section.isConnected && element("view").contains(section) && !element("view").inert;}
  /** Keep native focus while guarding duplicate or unacknowledged preparation and explicit read actions. */
  function actions() {
    exportButton.setAttribute("aria-disabled",String(!installed() || readOnly || busy || !sourceRestorePath(target.value) || !exportAck.checked));
    importButton.setAttribute("aria-disabled",String(!installed() || readOnly || busy || !file.files?.length || !["1","2"].includes(schema.value) || !indexAck.checked || !sourceAck.checked || !filesAck.checked));
    lookupButton.setAttribute("aria-disabled",String(!installed() || busy || !sourceRestoreId(lookup.value)));section.setAttribute("aria-busy",String(busy));
  }
  /** Every authored selection retires unsent work and requires fresh relevant acknowledgments. */
  function changed(event) {generation++;busy=false;error.hidden=true;if(event.target!==lookup)dirty=true;if(event.target===target)exportAck.checked=false;if(event.target===file || event.target===schema){indexAck.checked=false;sourceAck.checked=false;filesAck.checked=false;}actions();}
  /** Navigation/refresh/provenance retires all local acknowledgment and preparation ownership. */
  function invalidate() {generation++;busy=false;exportAck.checked=false;indexAck.checked=false;sourceAck.checked=false;filesAck.checked=false;actions();}
  /** Keep current local failure focus only when the invoker still owns it. */
  function localFailure(failure,invoker) {error.textContent=failure instanceof Error?failure.message:"The source preparation could not be started.";error.hidden=false;status.textContent="No source preparation was confirmed. Review the error and retry explicitly.";if(document.activeElement===invoker && !document.querySelector("dialog[open]"))error.focus();}
  /** Source export opts into exact bytes and reserves its complete planning target slot. */
  async function prepareExport() {
    if(!installed() || readOnly || busy || !sourceRestorePath(target.value) || !exportAck.checked)return;
    const sequence=++generation;const epoch=pending;const previewOwner=previewGeneration;const body={target_path:target.value,acknowledge_sensitive_metadata:true,acknowledge_source_content:true};busy=true;error.hidden=true;actions();
    /** This selection must still own the view after the shared request pacing wait. */
    const current=()=>installed() && sequence===generation && epoch===pending;
    try{await effect("/project/source-bundle-exports","POST",body,undefined,current,undefined,previewOwner);}catch(failure){if(current())localFailure(failure,exportButton);}finally{if(sequence===generation){busy=false;actions();}}
  }
  /** Preserve exact chosen JSON bytes and every explicit acknowledgment through delayed raw admission. */
  async function prepareImport() {
    if(!installed() || readOnly || busy || !file.files?.length || !["1","2"].includes(schema.value) || !indexAck.checked || !sourceAck.checked || !filesAck.checked)return;
    const selected=file.files[0];const targetVersion=Number(schema.value);const sequence=++generation;const epoch=pending;const previewOwner=previewGeneration;busy=true;error.hidden=true;actions();
    /** Retire old selected bytes after file changes, navigation, refresh or shutdown. */
    const current=()=>installed() && sequence===generation && epoch===pending;
    try{const body=await sourceBundleImportBody(selected,targetVersion);if(!current() || previewOwner!==previewGeneration)return;await effect("/project/source-bundle-imports","POST",undefined,body,current,targetVersion,previewOwner);}catch(failure){if(current())localFailure(failure,importButton);}finally{if(sequence===generation){busy=false;actions();}}
  }
  /** Look up one explicitly supplied public ID; no old receipt or session authority is revived. */
  async function lookupOutcome() {
    if(!installed() || busy || !sourceRestoreId(lookup.value))return;
    const row=sourceRestoreRow(lookup.value);const epoch=pending;const sequence=++generation;busy=true;error.hidden=true;actions();
    try{await pollSourceRestore(row,true);if(installed() && sequence===generation && epoch===pending && document.activeElement===lookupButton)row.status.focus();}catch(failure){if(installed() && sequence===generation && epoch===pending)localFailure(failure,lookupButton);}finally{if(sequence===generation){busy=false;actions();}}
  }
  const exportButton=node("button","Prepare source export");exportButton.type="button";exportButton.addEventListener("click",prepareExport);
  const importButton=node("button","Prepare source restore");importButton.type="button";importButton.addEventListener("click",prepareImport);
  const lookupButton=node("button","Look up source restore outcome");lookupButton.type="button";lookupButton.addEventListener("click",lookupOutcome);
  for(const control of [target,lookup])control.addEventListener("input",changed);
  for(const control of [exportAck,file,schema,indexAck,sourceAck,filesAck])control.addEventListener("change",changed);
  section.append(exportFields,exportButton,importFields,importButton,node("p",readOnly ? "Read-only session: source preparation/confirmation and cancellation are unavailable; known outcome lookup remains available." : "Exact source bundle bytes must fit1048429 bytes plus the147-byte acknowledged wrapper. Inline restore allows at most99 incoming resources with the index in the complete100-file union. Distinct export target plus present index allows at most98 registered resources."),lookupFields,lookupButton,node("p","After a lost reply or restart, launch a fresh same-project API2 2.3.0 session and explicitly look up the known ID. Not-found is not proof of no write and never authorizes a blind resend."),status,error);
  bundlePanels.set(section,invalidate);actions();return section;
}

/** Compare validated bounded DTO trees without treating JSON member order as semantic authority. */
function sourceRestoreEqual(left, right) {
  if(left===right)return true;
  if(!left || !right || typeof left!=="object" || typeof right!=="object" || Array.isArray(left)!==Array.isArray(right))return false;
  if(Array.isArray(left))return left.length===right.length && left.every((value,index)=>sourceRestoreEqual(value,right[index]));
  const keys=Object.keys(left).sort();const others=Object.keys(right).sort();
  return keys.length===others.length && keys.every((key,index)=>key===others[index] && sourceRestoreEqual(left[key],right[key]));
}
