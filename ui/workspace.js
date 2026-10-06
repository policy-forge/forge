// @ts-check
// The document contains no project data. All reads use the supported local API.
const element = (id) => document.getElementById(id);
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
const titles = ["Overview", "Review Queue", "Framework Scope", "Mappings", "Policies & Artifacts", "Trace & Reports"];

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

/** Preserve ordinary JSON requests while allowing one documented bounded raw-file query. */
async function api(path, method = "GET", body, key, rawBody, rawIsCurrent) {
  if (rawBody !== undefined && (path !== "/project/bundle-verifications" || method !== "POST" || !(rawBody instanceof Blob) || rawBody.size > 1024 * 1024)) throw new Error("Unsupported raw metadata comparison request.");
  const now=performance.now();const reserved=Math.max(now,nextRequestAt);nextRequestAt=reserved+60;
  if(reserved>now)await new Promise(resolve=>setTimeout(resolve,reserved-now));
  if (stopped) throw new Error("This workspace has stopped. Relaunch it from the terminal.");
  if (rawBody !== undefined && rawIsCurrent && !rawIsCurrent()) throw new Error("The metadata comparison was superseded before sending.");
  const headers = { "Accept": "application/json" };
  if (capability) headers["Authorization"] = `Bearer ${capability}`;
  if (method !== "GET") headers["Content-Type"] = "application/json";
  if (key) headers["Idempotency-Key"] = key;
  let response;
  try {
    response = await fetch(`/api/v1${path}`, {method, headers, body: method === "GET" ? undefined : rawBody ?? JSON.stringify(body ?? {}), cache: "no-store", credentials: "omit", redirect: "error", referrerPolicy: "no-referrer"});
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

/** Install only an owned live view; dismissed write previews cannot publish a late refresh. */
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
      fragment.append(visible.length ? table("Registered project files", [["Key","key"],["Role","role"],["Path","path"],["Validation","validation_state"]], visible) : node("p", "No matching registered files. Unregistered files are never scanned.", "empty"));
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
    } else {
      const resources = await collection("/resources");
      fragment.append(node("p", "Select a registered resource to inspect its source and decision references."));
      for (const resource of resources) fragment.append(button(`Trace ${resource.key}`, () => showProvenance(resource.resource_id)));
      if (!readOnly) fragment.append(exportForm());
      fragment.append(metadataBundlePanel());
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
    if (response.session.api_major !== 1) throw new Error("This UI requires API version 1. Install matching workspace assets.");
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
  try { await api("/session/shutdown", "POST", {}); stopped = true; invalidateBundlePanels(); pending++; capability = "";
    for (const row of operationRows.values()) { row.generation++; row.status.textContent = "Workspace stopped. Operation status is no longer queryable in this session."; row.cancel.setAttribute("aria-disabled", "true"); row.check.setAttribute("aria-disabled", "true"); row.review.disabled = true; }
    for (const row of requestRows.values()) { row.status.textContent = "Workspace stopped. The request cannot be recovered in this session."; row.retry.setAttribute("aria-disabled", "true"); row.review.setAttribute("aria-disabled", "true"); }
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

/** An action returning false cancelled navigation: reenable and refocus its invoker. */
function button(label, action) {
  const control = node("button", label); control.type = "button";
  control.addEventListener("click", async () => {
    control.disabled = true;
    let cancelled = false;
    try {cancelled = await action() === false;} catch(error) {showError(error);}
    finally {control.disabled = false;if(cancelled && control.isConnected)control.focus();}
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
  const row = { key, path, article, status, error, busy: false, prepared: null };
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
    const opened = await preview(row.prepared, undefined, () => !stopped && requestRows.get(key) === row && origin === pending && !document.querySelector("dialog[open]"));
    if (opened && !stopped && requestRows.get(key) === row) { row.error.hidden = true; row.status.textContent = "Preparation is ready for review; no write has been confirmed."; }
  }); row.review.hidden = true;
  article.append(node("h2", `Preparation request: ${path}`), status, error, row.retry, row.review); operationRegion().append(article); requestRows.set(key, row); return row;
}

/** Announce local recovery without changing a newer destination's global error or focus. */
function pendingRequestFailure(row, failure) {
  row.article.hidden = false; row.error.hidden = false; row.error.textContent = failure instanceof Error ? failure.message : "The preparation could not be read.";
  row.status.textContent = row.prepared ? "Preparation is ready. The prepared write could not be loaded; review it again before confirming." :
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
      const opened = await preview(result.preview || result.report_preview, path === "/exports" ? id : undefined, () => operationCurrent(row) && reviewOrigin === pending && !document.querySelector("dialog[open]"));
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

/** Accept this ID and state; report captured-registration facts without narrowing generic counter validation. */
function acceptOperation(row, value) {
  const expectedKind = { "/conversions": "conversion", "/applicability/analyses": "applicability-analysis", "/mapping/builds": "mapping-build", "/exports": "export" }[row.path];
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

/** Read known work with bounded polling; response loss retains an explicit GET recovery path. */
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
    if (operationCurrent(row) && row.last?.state === "succeeded" && row.origin === pending && !document.querySelector("dialog[open]")) {
      await preview(row.last.result.preview || row.last.result.report_preview, row.path === "/exports" ? row.id : undefined, () => operationCurrent(row) && row.origin === pending && !document.querySelector("dialog[open]"));
    }
  } catch (failure) {
    if (operationCurrent(row)) {
      if (row.last?.state === "succeeded" && row.terminal) operationPreviewFailure(row, failure); else operationUnknown(row, failure);
    }
  }
  finally { row.polling = false; if (operationCurrent(row)) refreshOperationActions(row); }
}

/** Preparation retries reuse one key until an ID is known; known work is recovered by GET. */
async function effect(path, method, request) {
  const key = crypto.randomUUID(); const origin = pending; let row; let recovery; let sending = false;
  /** Replay only an unacknowledged request; once acknowledged, query its existing operation. */
  const send = async () => {
    if (stopped) return;
    if (row) { await pollOperation(row, true); return; }
    const value = await api(path, method, request, key);
    if (stopped) return;
    if (value?.operation_id !== undefined || value?.state !== undefined) {
      if (typeof value.operation_id !== "string" || !/^op_[0-9a-z]{12,80}$/.test(value.operation_id)) {
        const unsupported = new Error("The operation returned an unsupported identity. Relaunch the workspace before continuing."); unsupported.details = { retryable: false }; throw unsupported;
      }
      row = operationRow(value.operation_id, path, origin); removePendingRequest(recovery, row.status);
      try {
        acceptOperation(row, value);
        if (!row.terminal) await pollOperation(row);
        else if (row.last?.state === "succeeded" && origin === pending && !document.querySelector("dialog[open]")) await preview(row.last.result.preview || row.last.result.report_preview, path === "/exports" ? row.id : undefined, () => operationCurrent(row) && origin === pending && !document.querySelector("dialog[open]"));
      } catch (failure) {
        if (operationCurrent(row)) {
          if (row.last?.state === "succeeded" && row.terminal) operationPreviewFailure(row, failure); else operationUnknown(row, failure);
        }
      }
      return;
    }
    recovery.prepared = value.preview || value.report_preview;
    if (!recovery.prepared?.preview_id) throw new Error("The operation did not return a prepared write.");
    const transferFocus = document.activeElement === recovery.retry;
    recovery.article.hidden = false; recovery.retry.hidden = true; recovery.review.hidden = false; recovery.error.hidden = true;
    recovery.status.textContent = "Preparation is ready for review; no write has been confirmed.";
    if (transferFocus && recovery.status.isConnected) recovery.status.focus();
    const opened = await preview(recovery.prepared, undefined, () => !stopped && origin === pending && !document.querySelector("dialog[open]"));
    if (opened) removePendingRequest(recovery);
  };
  /** Keep background failures local while preserving the original request's replay identity. */
  const attempt = async () => {
    if (sending || stopped) return;
    sending = true;
    try { await send(); } catch (failure) {
      if (stopped) return;
      pendingRequestFailure(recovery, failure);
      if (origin === pending && !recovery.busy) {
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

/** Open only the current receipt; confirmed writes own their refresh and native close focus. */
async function preview(proposed, exportOperation, isCurrent = () => !stopped) {
  if (!isCurrent()) return false;
  if(!proposed?.preview_id)throw new Error("The operation did not return a prepared write.");
  const sequence = ++previewGeneration;
  const current = await api(`/effects/previews/${encodeURIComponent(proposed.preview_id)}`);
  if (previewClosePending) await previewClosePending;
  if (!isCurrent() || sequence !== previewGeneration) return false;
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
  const key = crypto.randomUUID();
  /** Download only a confirmed export's bound bytes through the existing authenticated route. */
  async function downloadCommittedExport() {
    const response=await fetch(`/api/v1/exports/${encodeURIComponent(exportOperation)}/download`,{headers:{Authorization:`Bearer ${capability}`},cache:"no-store",credentials:"omit",redirect:"error",referrerPolicy:"no-referrer"});
    if(!response.ok)throw new Error("The committed export is no longer available or its bytes changed.");
    const blob=await response.blob();if(blob.size>4*1024*1024)throw new Error("The export exceeds the download bound.");
    const url=URL.createObjectURL(blob);const link=node("a","Download report");link.href=url;link.download="forge-redacted-report.html";document.body.append(link);link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
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
      if (exportOperation) element("view").prepend(button("Download committed redacted report", downloadCommittedExport));
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
function resourceActions(resources) {
  const region = node("section");region.append(node("h2","Project files"));
  region.append(button("Validate registered resources",async () => { const report = await api("/validation/runs","POST",{scope:"all"});element("status").textContent = `Validation: ${report.state}. ${report.error_count} errors.`; }));
  if(readOnly) {region.append(node("p","This session is read-only."));return region;}
  const roles = ["policy-source","oscal-catalog-artifact","oscal-component-artifact","mapping-collection","applicability-manifest","applicability-report","trace-report"].map(value=>[value,value.replaceAll("-"," ")]);
  const register = node("form");register.append(node("h3","Register an existing project file"));
  const role = field(register,"Resource role","select",roles);
  const path = field(register,"Project-relative file path");
  const key = field(register,"Stable resource key");
  register.append(button("Preview registration",async () => {if(!register.reportValidity())return;await effect("/resources/register","POST",{role:role.value,path:path.value,key:key.value});}));
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
  for (const section of element("view").querySelectorAll("[data-bundle-panel]")) bundlePanels.get(section)?.();
}

/** Accept only a supported closed response object, without dropping unknown metadata. */
function bundleClosedObject(value, keys) {
  return value !== null && typeof value === "object" && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
}

/** Validate complete metadata before retaining a local downloadable bundle. */
function checkedBundlePreview(value) {
  /** Reject unsupported preview metadata before it can become a local file. */
  const fail = () => { throw new Error("The metadata preview returned an unsupported response. Preview metadata again."); };
  if (!bundleClosedObject(value, ["bundle", "snapshot_version", "source_index_present", "included_metadata", "source_content_included"]) ||
      value.source_index_present !== true || value.source_content_included !== false ||
      typeof value.snapshot_version !== "string" || value.snapshot_version.length < 8 || value.snapshot_version.length > 128 ||
      JSON.stringify(value.included_metadata) !== JSON.stringify(["project-label", "resource-keys", "typed-roles", "project-relative-paths", "sha256-fingerprints", "byte-lengths"])) fail();
  const bundle = value.bundle;
  if (!bundleClosedObject(bundle, ["schema_version", "content_profile", "index", "index_sha256", "pins"]) ||
      bundle.schema_version !== "forge.workspace-index-bundle/1" || bundle.content_profile !== "index-and-hashes" ||
      typeof bundle.index_sha256 !== "string" || !/^[a-f0-9]{64}$/.test(bundle.index_sha256) ||
      !bundleClosedObject(bundle.index, ["schema_version", "label", "resources"]) || bundle.index.schema_version !== "forge.workspace/1" ||
      typeof bundle.index.label !== "string" || [...bundle.index.label].length < 1 || [...bundle.index.label].length > 200 ||
      !Array.isArray(bundle.index.resources) || bundle.index.resources.length > 1000 ||
      !Array.isArray(bundle.pins) || bundle.pins.length !== bundle.index.resources.length) fail();
  const keys = new Set(); const paths = new Set();
  const roles = ["policy-source", "oscal-catalog-artifact", "oscal-component-artifact", "mapping-collection", "applicability-manifest", "applicability-report", "trace-report"];
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
