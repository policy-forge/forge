// @ts-check
// The document contains no project data. All reads use the supported local API.
const element = (id) => document.getElementById(id);
let capability = "";
let activeView = "Overview";
let stopped = false;
let pending = 0;
let readOnly = true;
let dirty = false;
let nextRequestAt = 0;
let viewFilters = {};
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

async function api(path, method = "GET", body, key) {
  const now=performance.now();const reserved=Math.max(now,nextRequestAt);nextRequestAt=reserved+60;
  if(reserved>now)await new Promise(resolve=>setTimeout(resolve,reserved-now));
  if (stopped) throw new Error("This workspace has stopped. Relaunch it from the terminal.");
  const headers = { "Accept": "application/json" };
  if (capability) headers["Authorization"] = `Bearer ${capability}`;
  if (method !== "GET") headers["Content-Type"] = "application/json";
  if (key) headers["Idempotency-Key"] = key;
  let response;
  try {
    response = await fetch(`/api/v1${path}`, {method, headers, body: method === "GET" ? undefined : JSON.stringify(body ?? {}), cache: "no-store", credentials: "omit", redirect: "error", referrerPolicy: "no-referrer"});
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

async function pagedTable(path,caption,columns,filters=[]) {
  const section=node("section");const form=node("form");const display=node("div");const controls=node("div");
  const choices=filters.map(([key,label,values])=>{const select=field(form,label,"select",[["","All"],...values.map(value=>[value,value.replaceAll("-"," ")])]);select.required=false;if(viewFilters[key])select.value=viewFilters[key];return [key,select];});
  let cursors=[null];let index=0;
  const load=async()=>{
    const query=new URLSearchParams({page_size:"50"});for(const [key,select] of choices)if(select.value)query.set(key,select.value);
    if(cursors[index])query.set("cursor",cursors[index]);
    const response=await api(`${path}?${query}`);const page=response.page;
    display.replaceChildren(page.items.length?table(caption,columns,page.items):node("p","No matching items.","empty"),evidenceList(page.items));
    controls.replaceChildren(node("p",`${page.total_matching} matching items · Page ${index+1}`));
    if(index>0)controls.append(button("Previous page",async()=>{index--;await load();}));
    if(page.next_cursor)controls.append(button("Next page",async()=>{cursors[index+1]=page.next_cursor;index++;await load();}));
  };
  form.addEventListener("submit",event=>event.preventDefault());
  if(choices.length)form.append(button("Apply filters",async()=>{cursors=[null];index=0;await load();}));
  section.append(form,display,controls);await load();return section;
}

async function renderView() {
  const sequence = ++pending;
  element("error").hidden = true;
  element("status").textContent = "Loading project state…";
  element("refresh").disabled = true;
  const fragment = document.createDocumentFragment();
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
        const card=button("",()=>navigate(destination,filters));card.className="card";
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
    }
    if (sequence !== pending) return;
    element("view").replaceChildren(fragment);
    element("view-title").textContent = activeView;
    element("status").textContent = "Project state loaded.";
  } catch (error) { if (sequence === pending) { element("view").replaceChildren(); showError(error); element("status").textContent = "The view could not be loaded."; } }
  finally { if (sequence === pending) element("refresh").disabled = false; }
}

element("unlock-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  const button = event.currentTarget.querySelector("button");
  button.disabled = true;
  try {
    if ([...element("passphrase").value].length < 15 || [...element("passphrase").value].length > 128) throw new Error("Use 15–128 characters.");
    const response = await api("/session/unlock", "POST", {passphrase: element("passphrase").value});
    element("passphrase").value = "";
    if (response.session.api_major !== 1) throw new Error("This UI requires API version 1. Install matching workspace assets.");
    capability = response.capability;
    readOnly = response.session.read_only;
    element("unlock-panel").hidden = true;
    element("workspace").hidden = false;
    element("stop").hidden = false;
    element("connection").textContent = response.session.read_only ? "Read-only · Local" : "Local session";
    await renderView(); element("main").focus();
  } catch (error) { element("passphrase").value = ""; showError(error); }
  finally { button.disabled = false; }
});

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
  try { await api("/session/shutdown", "POST", {}); stopped = true; capability = ""; element("stop-dialog").close(); element("workspace").hidden = true; element("stop").hidden = true; element("connection").textContent = "Stopped"; element("status").textContent = "Workspace stopped. Relaunch it from the terminal to continue."; element("main").focus(); }
  catch (error) { element("stop-dialog").close(); showError(error); }
});


async function navigate(title, filters = {}) {
  if (dirty && !await confirmDiscard()) return;
  dirty = false; activeView = title; viewFilters = filters;
  for (const sibling of element("navigation").children) {
    if (sibling.textContent === title) sibling.setAttribute("aria-current", "page"); else sibling.removeAttribute("aria-current");
  }
  await renderView();element("view-title").tabIndex=-1;element("view-title").focus();
}
function confirmDiscard() {
  const dialog=node("dialog");dialog.setAttribute("aria-labelledby","discard-title");
  const heading=node("h2","Discard unconfirmed edits?");heading.id="discard-title";
  dialog.append(heading,node("p","Only this page's unconfirmed form changes will be discarded. Saved files remain unchanged."));
  return new Promise(resolve=>{
    let confirmed=false;dialog.append(button("Keep editing",()=>dialog.close()),button("Discard edits",()=>{confirmed=true;dialog.close();}));
    dialog.addEventListener("close",()=>{dialog.remove();resolve(confirmed);},{once:true});document.body.append(dialog);dialog.showModal();
  });
}
function button(label, action) {
  const control = node("button", label); control.type = "button";
  control.addEventListener("click", async () => { control.disabled = true; try {await action();} catch(error) {showError(error);} finally {control.disabled = false;} });
  return control;
}
function field(form, label, type = "text", options) {
  const id = `field-${crypto.randomUUID()}`;
  const caption = node("label", label); caption.htmlFor = id;
  const input = node(type === "select" ? "select" : type === "textarea" ? "textarea" : "input");
  input.id = id; input.required = true;
  if (type !== "select" && type !== "textarea") input.type = type;
  if (type === "textarea") {input.rows = 18; input.spellcheck = false;}
  if (options) for (const [value,label] of options) {const option = node("option",label);option.value = value;input.append(option);}
  input.addEventListener("input", () => {dirty = true;});
  form.append(caption,input); return input;
}
async function effect(path, method, request) {
  const key = crypto.randomUUID();
  const send = async () => {
    let value = await api(path, method, request, key);
    const operationId=value.operation_id;
    if (value.operation_id && value.state) {
      element("status").textContent = `Operation ${value.state}.`;
      const cancel=button("Cancel pending operation",()=>api(`/operations/${encodeURIComponent(operationId)}/cancellation`,"POST",{}));
      element("status").append(cancel);
      try {
        while (value.state === "pending" || value.state === "running") {
          await new Promise(resolve => setTimeout(resolve,500));
          value = await api(`/operations/${encodeURIComponent(operationId)}`);
        element("connection").textContent=`Local · Operation ${value.state}`;
        }
      } finally {cancel.remove();}
      if (value.state !== "succeeded") {const error=new Error(value.error?.message || `Operation ${value.state}.`);error.details=value.error||{retryable:false};throw error;}
      value = value.result;
    }
    await preview(value.preview || value.report_preview,path === "/exports" ? operationId : undefined);
  };
  try {await send();} catch(error) {showError(error);if(error.details?.retryable!==false)element("error").append(button("Retry the same request",send));}
}
async function preview(proposed, exportOperation) {
  if(!proposed?.preview_id)throw new Error("The operation did not return a prepared write.");
  const current = await api(`/effects/previews/${encodeURIComponent(proposed.preview_id)}`);
  const dialog = element("preview-dialog"); const content = element("preview-content");
  content.replaceChildren(Object.assign(node("h2", "Review proposed write"),{id:"preview-title"}),node("p", `${current.target.status}: ${current.target.path}`),node("p", current.semantic_summary),
    node("p", `Validation: ${current.validation.state}. Target version: ${current.target_version}`),node("p", `Current hash: ${current.base_sha256 || "new file"}`),node("p", `Proposed hash: ${current.exact_bytes_sha256}`),node("p", `Receipt expires: ${current.receipt.expires_at}`),
    table("Bound input hashes",[["Resource","resource_id"],["SHA-256","sha256"]],current.input_hashes),Object.assign(node("pre",current.diff_text),{tabIndex:0}));
  if(current.diff_truncated) content.append(node("p","The text diff reached its display bound. The hash binds the complete proposed bytes."));
  const key = crypto.randomUUID();
  content.append(button("Keep editing", () => dialog.close()), button("Confirm this exact write", async () => {
    const operation = await api("/effects/commits","POST",{receipt:current.receipt.token,observed_version:current.target_version,confirmed:true},key);
    const observed = await api(`/operations/${encodeURIComponent(operation.operation_id)}`);
    if(observed.state !== "succeeded") throw new Error(observed.error?.message || "The write has not completed.");
    dirty = false;dialog.close();await renderView();element("status").textContent = `Saved ${observed.result.target_path}.`;
    if(exportOperation) element("view").prepend(button("Download committed redacted report",async()=>{
      const response=await fetch(`/api/v1/exports/${encodeURIComponent(exportOperation)}/download`,{headers:{Authorization:`Bearer ${capability}`},cache:"no-store",credentials:"omit",redirect:"error",referrerPolicy:"no-referrer"});
      if(!response.ok)throw new Error("The committed export is no longer available or its bytes changed.");
      const blob=await response.blob();if(blob.size>4*1024*1024)throw new Error("The export exceeds the download bound.");
      const url=URL.createObjectURL(blob);const link=node("a","Download report");link.href=url;link.download="forge-redacted-report.html";document.body.append(link);link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);
    }));
  }));
  dialog.showModal();content.querySelector("h2").tabIndex=-1;content.querySelector("h2").focus();
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
  try {draft=await api(`/${kind}/draft`);} catch {return initializeForm(kind);}
  const form=node("form");
  form.append(node("h2",kind==="mapping"?"Explicit mapping decisions":"Explicit applicability decisions"),node("p","Edit the complete decision document. Supply the reviewer key, review time, state or relationship, and rationale explicitly. Reviewer metadata is asserted provenance."));
  const input=field(form,"Decision manifest (JSON)","textarea");input.value=JSON.stringify(draft.manifest,null,2);input.readOnly=readOnly;
  if(kind==="applicability"&&!readOnly)form.append(await applicabilityDecisionForm(input));
  if(kind==="mapping"&&!readOnly)form.append(mappingDecisionForm(input));
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

function mappingDecisionForm(input) {
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
  }));return section;
}

function evidenceList(rows) {
  const list=node("section");list.append(node("h2","Review evidence"));
  for(const row of rows)for(const anchor of row.evidence_refs||(row.provenance_ref?[row.provenance_ref]:[]))list.append(button(`Inspect ${row.control_id||row.label||row.reason_code}`,()=>showProvenance(anchor)));
  return list;
}
async function showProvenance(anchor) {
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
  element("view").replaceChildren(region);element("view-title").textContent="Provenance";element("main").focus();
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
