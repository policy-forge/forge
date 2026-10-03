// Execute the entire real source asset, never extracted or rewritten functions.
// FORGE_TEST_WORKSPACE_JS selects an actual proposal path for development only;
// absent that override, this tests the deployed asset beside the checked-in file.
// V8 filenames and the diagnostic identify that exact file and SHA-256.
// This fake DOM is source-level evidence, not browser, layout, keyboard or AT evidence.
// Native close events are queued; controlled timers model polling without real sleep.
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { randomUUID, createHash, webcrypto } = require("node:crypto");
const { Blob } = require("node:buffer");

const productionPath = process.env.FORGE_TEST_WORKSPACE_JS
  ? path.resolve(process.env.FORGE_TEST_WORKSPACE_JS) : path.resolve(__dirname, "../workspace.js");
const productionSource = fs.readFileSync(productionPath, "utf8");
const sourceSha256 = createHash("sha256").update(productionSource).digest("hex");
test("source receipt binds complete executed file", context => {
  context.diagnostic(JSON.stringify({source_path: productionPath, source_sha256: sourceSha256,
    source_scope: process.env.FORGE_TEST_WORKSPACE_JS ? "explicit-source-path" : "adjacent-worktree-asset"}));
  assert(productionSource.startsWith("// @ts-check"));
});

/** Construct a cancellable DOM event for the narrow interaction harness. */
function eventFor(type, target) {
  return {
    type, target, currentTarget: target, defaultPrevented: false,
    /** Retain cancellation so dispatch can distinguish an accepted default action. */
    preventDefault() { this.defaultPrevented = true; },
  };
}

/** Minimal DOM node preserving ownership, focus eligibility, and event lifecycle. */
class FakeNode {
  /** Create a node owned by one isolated fake document. */
  constructor(document, tagName) {
    this.ownerDocument = document;
    this.tagName = tagName.toUpperCase();
    this.children = [];
    this.parentNode = null;
    this.attributes = new Map();
    this.listeners = new Map();
    this.id = "";
    this.className = "";
    this.value = "";
    this.files = [];
    this.checked = false;
    this._hidden = false;
    this.open = false;
    this.required = false;
    this.tabIndex = undefined;
    this._text = "";
    this.textWrites = [];
    this._disabled = false;
    this._inert = false;
  }

  /** A detached node cannot receive focus or serve as a dialog return target. */
  get isConnected() {
    let current = this;
    while (current.parentNode) current = current.parentNode;
    return current === this.ownerDocument.body;
  }

  /** Expose display removal used by persistent recovery and shutdown transitions. */
  get hidden() { return this._hidden; }

  /** Hiding a focused subtree drops focus; invisible recovery targets cannot remain active. */
  set hidden(value) {
    this._hidden = Boolean(value);
    if (this._hidden && this.contains(this.ownerDocument.activeElement)) {
      this.ownerDocument.activeElement = this.ownerDocument.body;
    }
  }

  /** Expose the disabled state used by production cancellation recovery. */
  get disabled() { return this._disabled; }

  /** Disabling the focused control drops its active focus. */
  set disabled(value) {
    this._disabled = Boolean(value);
    if (this._disabled && this.ownerDocument.activeElement === this) {
      this.ownerDocument.activeElement = this.ownerDocument.body;
    }
  }

  /** Expose inert view state while a destination is being prepared. */
  get inert() { return this._inert; }

  /** Inert subtrees cannot retain active interactive focus. */
  set inert(value) {
    this._inert = Boolean(value);
    if (this._inert && this.contains(this.ownerDocument.activeElement)) {
      this.ownerDocument.activeElement = this.ownerDocument.body;
    }
  }

  /** Return the same descendant text production uses for navigation labels. */
  get textContent() {
    return this._text + this.children.map(child => child.textContent).join("");
  }

  /** Replace descendants and record each live-region write, including repeated announcements. */
  set textContent(value) {
    this.replaceChildren();
    this._text = String(value);
    this.textWrites.push(this._text);
  }

  /** Append nodes and flatten document fragments as the DOM does. */
  append(...nodes) {
    for (const child of nodes) {
      if (child.tagName === "#FRAGMENT") {
        this.append(...[...child.children]);
      } else {
        if (child.parentNode) child.remove();
        child.parentNode = this;
        this.children.push(child);
      }
    }
  }

  /** Prepend nodes while preserving their supplied order. */
  prepend(...nodes) {
    const previous = [...this.children];
    this.children = [];
    for (const child of previous) child.parentNode = null;
    this.append(...nodes, ...previous);
  }

  /** Replace a region and detach its previous form and focus targets. */
  replaceChildren(...nodes) {
    for (const child of [...this.children]) child.remove();
    this.children = [];
    this._text = "";
    this.append(...nodes);
  }

  /** Detach a subtree and clear focus if its active descendant was removed. */
  remove() {
    if (this.contains(this.ownerDocument.activeElement)) {
      this.ownerDocument.activeElement = this.ownerDocument.body;
    }
    if (this.parentNode) {
      const siblings = this.parentNode.children;
      siblings.splice(siblings.indexOf(this), 1);
      this.parentNode = null;
    }
  }

  /** Test ancestry for inertness, connection, and modal focus constraints. */
  contains(candidate) {
    while (candidate) {
      if (candidate === this) return true;
      candidate = candidate.parentNode;
    }
    return false;
  }

  /** Record attributes needed by the actual UI's semantics and selectors. */
  setAttribute(name, value) {
    this.attributes.set(name, String(value));
    if (name === "id") this.id = String(value);
    if (name === "tabindex") this.tabIndex = Number(value);
  }

  /** Return a recorded attribute or the DOM's absent-attribute value. */
  getAttribute(name) { return this.attributes.get(name) ?? null; }

  /** Remove an attribute, including aria-current on navigation controls. */
  removeAttribute(name) { this.attributes.delete(name); }

  /** Support the simple tag/attribute selectors used by production functions. */
  matches(selector) {
    const match = selector.trim().match(/^([a-z0-9-]+)?(?:\[([a-z0-9-]+)(?:=["']?([^"'\]]+)["']?)?\])?$/i);
    if (!match) throw new Error("Unsupported fake DOM selector: " + selector);
    const [, tag, attribute, value] = match;
    if (tag && this.tagName !== tag.toUpperCase()) return false;
    if (!attribute) return true;
    if (attribute === "open") return this.open;
    if (attribute === "tabindex") return this.tabIndex !== undefined;
    if (value === undefined) return this.attributes.has(attribute);
    return this.attributes.get(attribute) === value;
  }

  /** Find descendants in document order, including comma-separated selectors. */
  querySelectorAll(selector) {
    const selectors = selector.split(",");
    const result = [];
    /** Walk owned descendants in the same order used by selector assertions. */
    const visit = node => {
      for (const child of node.children) {
        if (selectors.some(part => child.matches(part))) result.push(child);
        visit(child);
      }
    };
    visit(this);
    return result;
  }

  /** Return the first matching descendant. */
  querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }

  /** Install a listener and retain its once semantics for dialog dismissal. */
  addEventListener(type, callback, options = {}) {
    const listeners = this.listeners.get(type) ?? [];
    listeners.push({ callback, once: Boolean(options.once) });
    this.listeners.set(type, listeners);
  }

  /** Remove exactly one callback, including native-preview lifecycle listeners. */
  removeEventListener(type, callback) {
    const listeners = this.listeners.get(type) ?? [];
    this.listeners.set(type, listeners.filter(listener => listener.callback !== callback));
  }

  /** Dispatch synchronously; retain returned promises for explicit test waiting. */
  dispatchEvent(event) {
    event.currentTarget = this;
    const results = [];
    for (const listener of [...(this.listeners.get(event.type) ?? [])]) {
      if (listener.once) {
        const listeners = this.listeners.get(event.type);
        listeners.splice(listeners.indexOf(listener), 1);
      }
      results.push(listener.callback.call(this, event));
    }
    event.results = results;
    return !event.defaultPrevented;
  }

  /** Activate real production listeners and await their asynchronous completion. */
  async fire(type) {
    const event = eventFor(type, this);
    this.dispatchEvent(event);
    await Promise.all(event.results);
    return event;
  }

  /** Refuse focus on disabled, disconnected, hidden, inert, or nonfocusable nodes. */
  focus() {
    if (!this.isConnected || this.disabled) return;
    for (let ancestor = this; ancestor; ancestor = ancestor.parentNode) {
      if (ancestor.hidden || ancestor.inert) return;
    }
    const modal = this.ownerDocument.querySelector("dialog[open]");
    if (modal && !modal.contains(this)) return;
    const native = ["BUTTON", "INPUT", "SELECT", "TEXTAREA", "A"].includes(this.tagName);
    if (!native && this.tabIndex === undefined) return;
    this.ownerDocument.activeElement = this;
    this.ownerDocument.focusHistory.push(this);
  }

  /** Model native modal activation; production must deliberately choose its target. */
  showModal() {
    this.returnTarget = this.ownerDocument.activeElement;
    this.open = true;
    this.ownerDocument.activeElement = this;
  }

  /** Queue the native close event after restoration; close listeners are not synchronous. */
  close() {
    if (!this.open) return;
    this.open = false;
    this.ownerDocument.activeElement = this.ownerDocument.body;
    this.returnTarget?.focus();
    /** Dispatch this queued native close only when its controlled lifecycle is released. */
    const emit = () => this.dispatchEvent(eventFor("close", this));
    if (this.ownerDocument.holdCloseEvents) this.ownerDocument.closeEvents.push(emit);
    else setImmediate(emit);
  }

  /** Escape emits cancel and uses native non-destructive dismissal unless prevented. */
  escape() {
    const event = eventFor("cancel", this);
    if (this.dispatchEvent(event)) this.close();
  }

  /** Observe local anchor dispatch; this double is not native browser download evidence. */
  click() {
    if (this.tagName === "A") this.ownerDocument.downloads.push({url:this.href,filename:this.download});
    else this.dispatchEvent(eventFor("click",this));
  }

  /** These tests use valid values; constraint validation is outside this harness. */
  reportValidity() { return true; }
}

/** Construct the static shell nodes production initializes on module load. */
class FakeDocument {
  /** Assemble unchanged body targets and explicit server-selected head metadata before asset initialization. */
  constructor(bootstrap = {"forge-api-major":"1"}) {
    this.head = new FakeNode(this, "head");
    for (const [name, content] of Object.entries(bootstrap)) {
      const meta = this.createElement("meta");
      meta.setAttribute("name", name);
      if (content !== null) meta.setAttribute("content", content);
      this.head.append(meta);
    }
    this.body = new FakeNode(this, "body");
    this.activeElement = this.body;
    this.focusHistory = [];
    this.downloads = [];
    this.holdCloseEvents = false;
    this.closeEvents = [];
    /** Connect one static shell element before production initializes its handlers. */
    const add = (parent, tag, id, text = "") => {
      const node = this.createElement(tag);
      node.id = id;
      node.textContent = text;
      parent.append(node);
      return node;
    };
    const main = add(this.body, "main", "main");
    main.tabIndex = -1;
    const unlock = add(main, "section", "unlock-panel");
    const unlockForm = add(unlock, "form", "unlock-form");
    add(unlockForm, "input", "passphrase");
    add(unlockForm, "button", "", "Unlock workspace");
    const workspace = add(main, "section", "workspace");
    add(workspace, "nav", "navigation");
    add(workspace, "h1", "view-title", "Overview");
    add(workspace, "button", "refresh", "Refresh");
    add(workspace, "div", "view");
    const error = add(main, "div", "error");
    error.setAttribute("role", "alert");
    error.tabIndex = -1;
    error.hidden = true;
    const status = add(main, "p", "status");
    status.setAttribute("role", "status");
    add(this.body, "span", "connection", "Local session");
    add(this.body, "button", "stop", "Stop workspace");
    const preview = add(main, "dialog", "preview-dialog");
    add(preview, "div", "preview-content");
    const stop = add(main, "dialog", "stop-dialog");
    add(stop, "button", "keep-working", "Keep working");
    add(stop, "button", "confirm-stop", "Stop workspace");
  }

  /** Allocate an element with a shared owning document. */
  createElement(tag) { return new FakeNode(this, tag); }

  /** Allocate an unconnected fragment flattened by append and replaceChildren. */
  createDocumentFragment() { return this.createElement("#fragment"); }

  /** Resolve IDs from connected nodes only, as real document lookup does. */
  getElementById(id) {
    /** Search only the connected tree so detached recovery targets are absent. */
    const visit = node => {
      if (node.id === id) return node;
      for (const child of node.children) {
        const found = visit(child);
        if (found) return found;
      }
      return null;
    };
    return visit(this.body);
  }

  /** Resolve declared head metadata before body modal targets; metadata never joins the focus tree. */
  querySelector(selector) { return this.head.querySelector(selector) ?? this.body.querySelector(selector); }
}

/** Execute the unchanged whole asset with explicit shell metadata and synthetic JSON or binary transport fixtures. */
function harness(overrides = {}, bootstrap = {"forge-api-major":"1"}) {
  const document = new FakeDocument(bootstrap);
  const requests = [];
  const objectURLs = new Map(); const revokedURLs = [];
  /** Track Blob URL lifetimes without mutating the host URL implementation. */
  class HarnessURL extends URL {
    /** Retain the actual Blob bytes for source-level local-download assertions. */
    static createObjectURL(blob) { const key="blob:synthetic-"+randomUUID(); objectURLs.set(key,blob);return key; }
    /** Record retirement of a previously retained local Blob URL. */
    static revokeObjectURL(key) { objectURLs.delete(key);revokedURLs.push(key); }
  }
  const routes = {
    "/project/summary": () => ({
      version: "synthetic-version-1", health: "ready",
      project_label: "Synthetic project", workspace_index_present: true,
      resource_counts: { total: 1, valid: 1, invalid: 0, stale: 0, not_validated: 0 },
      review_counts: { total_open: 0 }, next_action: "review-scope",
    }),
    "/project/config-status": () => ({ present: true, valid: true }),
    "/resources": () => ({ resource_version: "synthetic-version-1", page: { items: [], total_matching: 0, next_cursor: null } }),
    "/provenance/entries": () => ({ resource_version: "synthetic-version-1", page: { items: [], total_matching: 0, next_cursor: null } }),
    ...overrides,
  };
  let clock = 0;
  const timers = [];
  const context = vm.createContext({
    document,
    window: {
      /** Ignore window-level subscriptions; this source harness does not dispatch unload. */
      addEventListener() {}
    },
    crypto: { randomUUID, subtle:webcrypto.subtle },
    performance: { now: () => (clock += 100) },
    setTimeout: (callback, milliseconds = 0) => {
      if (milliseconds >= 500) timers.push(callback);
      else queueMicrotask(callback);
      return timers.length;
    },
    URLSearchParams, URL:HarnessURL, Blob, console,
    /** Record exact selected-major URLs while substituting only authored JSON or binary transport observations. */
    fetch: async (url, options) => {
      const parsed = new URL(url, "http://127.0.0.1:1");
      const route = parsed.pathname.replace(/^\/api\/v[12](?=\/)/, "");
      requests.push({ route, url, options });
      assert.equal(typeof routes[route], "function", "Unexpected API route: " + route);
      const value = await routes[route](parsed, options);
      const status = value.status ?? 200;
      return { ok: status >= 200 && status < 300, json: async () => value.body ?? value,
        headers: { /** Expose only explicitly authored media metadata for exact binary-download probes. */ get: name => name.toLowerCase() === "content-type" ? value.download_media_type ?? null : null },
        /** Supply only an explicitly authored binary download response; ordinary JSON remains unchanged. */
        blob: async () => { assert(value.download_blob instanceof Blob, "Missing binary download fixture"); return value.download_blob; } };
    },
  });
  new vm.Script(productionSource, { filename: productionPath }).runInContext(context);
  /** Evaluate test interactions within the isolated realm that owns the full asset. */
  const run = source => vm.runInContext(source, context);
  /** Resolve a currently connected production control by its shell identifier. */
  const byId = id => document.getElementById(id);
  /** Require the named button in the intended region before activating it. */
  const byButton = (label, root = document.body) => {
    const control = root.querySelectorAll("button").find(node => node.textContent === label);
    assert(control, "Missing button: " + label);
    return control;
  };
  /** Resolve a labelled control through the actual rendered label association. */
  const byLabel = label => {
    const caption = document.body.querySelectorAll("label").find(node => node.textContent === label);
    assert(caption, "Missing label: " + label);
    return byId(caption.htmlFor);
  };
  /** Advance one real production polling delay, retaining control over in-flight reads. */
  const poll = async () => {
    assert(timers.length, "No pending polling delay");
    timers.shift()();
    await settle();
  };
  return { document, requests, routes, run, byId, byButton, byLabel, poll, timers, objectURLs, revokedURLs };
}

/** Let event callbacks reach their first awaited boundary without wall-clock sleeps. */
async function settle() { await new Promise(resolve => setImmediate(resolve)); }

/** Create a controllable in-flight API response for stale-result tests. */
function deferred() {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
}

/** Install actual registration and evidence controls and mark actual fields dirty. */
async function unsavedForm(app) {
  app.run('readOnly = false; element("view").replaceChildren(evidenceList([{label:"Fixture policy",provenance_ref:"prov_synthetic000001"}]),resourceActions([]));');
  const input = app.byLabel("Project-relative file path");
  input.value = "unsaved-policy.md";
  await input.fire("input");
  const key = app.byLabel("Stable resource key");
  key.value = "unsaved-key";
  await key.fire("input");
  assert.equal(app.run("dirty"), true);
  return { input, key, inspect: app.byButton("Inspect Fixture policy") };
}

/** Assert discard cancellation retains the same connected form and its exact values. */
function retained(app, fields) {
  assert.equal(fields.input.isConnected, true);
  assert.equal(fields.key.isConnected, true);
  assert.equal(fields.input.value, "unsaved-policy.md");
  assert.equal(fields.key.value, "unsaved-key");
  assert.equal(app.run("dirty"), true);
}

test("successful clean navigation focuses the committed destination", async () => {
  const app = harness();
  const result = await app.run('navigate("Trace & Reports")');
  assert.notEqual(result, false);
  assert.equal(app.byId("view-title").textContent, "Trace & Reports");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.byId("status").textContent, "Project state loaded.");
});

test("navigation API failure retains focused error summary rather than stale title", async () => {
  const app = harness({ "/resources": () => ({
    status: 500, body: { code: "internal-error", message: "Synthetic navigation failure.", retryable: true },
  }) });
  await app.run('navigate("Trace & Reports")');
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.equal(app.byId("error").hidden, false);
  assert.match(app.byId("error").textContent, /internal-error.*Synthetic navigation failure/);
  assert.equal(app.byId("view-title").textContent, "Overview");
});

test("cancelled dirty Inspect preserves values, dirtiness, focus, and avoids API reads", async () => {
  const app = harness();
  const fields = await unsavedForm(app);
  fields.inspect.focus();
  const activation = fields.inspect.fire("click");
  await settle();
  const dialog = app.document.querySelector("dialog[open]");
  assert(dialog);
  assert.equal(app.document.activeElement, dialog.querySelector("h2"));
  await app.byButton("Keep editing", dialog).fire("click");
  await activation;
  retained(app, fields);
  assert.equal(app.requests.length, 0);
  assert.equal(fields.inspect.disabled, false);
  assert.equal(app.document.activeElement, fields.inspect);
});

test("confirmed dirty Inspect clears dirtiness only on a successful destination", async () => {
  const app = harness();
  const fields = await unsavedForm(app);
  const activation = fields.inspect.fire("click");
  await settle();
  await app.byButton("Discard edits", app.document.querySelector("dialog[open]")).fire("click");
  await activation;
  assert.equal(app.run("dirty"), false);
  assert.equal(fields.input.isConnected, false);
  assert.equal(app.byId("view-title").textContent, "Provenance");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.match(app.byId("status").textContent, /provenance.*loaded/i);
  assert.equal(app.requests.length, 1);
});

test("dirty provenance failure preserves the original form and focused API error", async () => {
  const app = harness({ "/provenance/entries": () => ({
    status: 500, body: { code: "internal-error", message: "Synthetic provenance failure.", retryable: true },
  }) });
  const fields = await unsavedForm(app);
  const activation = fields.inspect.fire("click");
  await settle();
  await app.byButton("Discard edits", app.document.querySelector("dialog[open]")).fire("click");
  await activation;
  retained(app, fields);
  assert.equal(app.byId("view").inert, false);
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.match(app.byId("error").textContent, /Synthetic provenance failure/);
});

test("Escape cancels dirty Inspect and restores the connected initiating control", async () => {
  const app = harness();
  const fields = await unsavedForm(app);
  const activation = fields.inspect.fire("click");
  await settle();
  app.document.querySelector("dialog[open]").escape();
  await activation;
  retained(app, fields);
  assert.equal(app.requests.length, 0);
  assert.equal(app.document.activeElement, fields.inspect);
});

test("direct discard dismissal restores its enabled invoking control", async () => {
  const app = harness();
  const invoker = app.byId("refresh");
  invoker.focus();
  const decision = app.run("confirmDiscard()");
  const dialog = app.document.querySelector("dialog[open]");
  assert.equal(app.document.activeElement, dialog.querySelector("h2"));
  dialog.escape();
  assert.equal(await decision, false);
  assert.equal(app.document.activeElement, invoker);
});

test("Trace view's actual button loads provenance and focuses its destination", async () => {
  const app = harness({ "/resources": () => ({ page: {
    items: [{ resource_id: "res_synthetic000001", key: "fixture-policy", role: "policy-source", path: "synthetic-policy.md", sha256: "a".repeat(64), size_bytes: 1, validation_state: "valid", stale: false, version: "synthetic-version-1" }],
    total_matching: 1, next_cursor: null,
  } }) });
  await app.run('navigate("Trace & Reports")');
  await app.byButton("Trace fixture-policy").fire("click");
  assert.equal(app.byId("view-title").textContent, "Provenance");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.match(app.requests.at(-1).url, /anchor=res_synthetic000001/);
});

test("a superseded render returns false and cannot steal destination focus", async () => {
  const response = deferred();
  const app = harness({ "/resources": () => response.promise });
  app.run('activeView = "Trace & Reports";');
  const stale = app.run("renderView()");
  await settle();
  assert.equal(app.byId("view").inert, true);
  await app.run('navigate("Overview")');
  const focusCount = app.document.focusHistory.length;
  response.resolve({ page: { items: [], total_matching: 0, next_cursor: null } });
  assert.equal(await stale, false);
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
});

test("pending provenance makes old inputs inert and ignores a superseded result", async () => {
  const response = deferred();
  const app = harness({ "/provenance/entries": () => response.promise });
  const input = app.document.createElement("input");
  app.byId("view").append(input);
  const stale = app.run('showProvenance("prov_synthetic000001")');
  await settle();
  assert.equal(app.byId("view").inert, true);
  input.focus();
  assert.notEqual(app.document.activeElement, input);
  await app.run('navigate("Overview")');
  const focusCount = app.document.focusHistory.length;
  response.resolve({ page: { items: [], total_matching: 0, next_cursor: null } });
  await stale;
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("view").inert, false);
});

test("a superseded provenance failure cannot announce an obsolete error", async () => {
  const response = deferred();
  const app = harness({ "/provenance/entries": () => response.promise });
  const stale = app.run('showProvenance("prov_synthetic000001")');
  await settle();
  assert.equal(app.byId("view").inert, true);
  assert.equal(app.byId("refresh").disabled, true);
  await app.run('navigate("Overview")');
  const focusCount = app.document.focusHistory.length;
  response.resolve({ status: 500, body: {
    code: "internal-error", message: "Obsolete provenance failure.", retryable: true,
  } });
  await stale;
  assert.equal(app.byId("error").hidden, true);
  assert.equal(app.byId("status").textContent, "Project state loaded.");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("view").inert, false);
  assert.equal(app.byId("refresh").disabled, false);
});

for (const unavailable of ["removed", "disabled"]) {
  test("discard dismissal cannot restore a " + unavailable + " invoker", async () => {
    const app = harness();
    const invoker = app.byId("refresh");
    invoker.focus();
    const decision = app.run("confirmDiscard()");
    const dialog = app.document.querySelector("dialog[open]");
    if (unavailable === "removed") invoker.remove();
    else invoker.disabled = true;
    dialog.escape();
    assert.equal(await decision, false);
    assert.notEqual(app.document.activeElement, invoker);
    assert.equal(app.document.querySelector("dialog[open]"), null);
  });
}

test("initial unlock preserves a project-load failure's error-summary focus", async () => {
  const app = harness({
    "/session/unlock": () => ({
      capability: "synthetic-test-capability-32-characters",
      session: { session_id: "sess_synthetic001", mode: "browser", api_major: 1, read_only: true, contract_version: "1.0.0", project_label: "Synthetic project" },
    }),
    "/project/summary": () => ({ status: 500, body: {
      code: "internal-error", message: "Synthetic initial-load failure.", retryable: true,
    } }),
  });
  app.byId("passphrase").value = "synthetic test passphrase";
  const form = app.byId("unlock-form");
  await form.fire("submit");
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.match(app.byId("error").textContent, /Synthetic initial-load failure/);
  assert.equal(app.byId("passphrase").value, "");
  assert.equal(form.querySelector("button").disabled, false);
  assert.equal(app.byId("view-title").textContent, "Overview");
});

/** Build the actual Trace/export view and dirty its report destination field. */
async function dirtyTraceForm(app) {
  app.run("readOnly = false;");
  await app.run('navigate("Trace & Reports")');
  const target = app.byLabel("Report destination within project");
  target.value = "unsaved-report.html";
  await target.fire("input");
  assert.equal(app.run("dirty"), true);
  return { target, trace: app.byButton("Trace fixture-policy") };
}

for (const discard of [false, true]) {
  test("dirty Trace callback " + (discard ? "discards explicitly" : "preserves export values on cancellation"), async () => {
    const app = harness({ "/resources": () => ({ page: {
      items: [{ resource_id: "res_synthetic000001", key: "fixture-policy", role: "policy-source", path: "synthetic-policy.md", sha256: "a".repeat(64), size_bytes: 1, validation_state: "valid", stale: false, version: "synthetic-version-1" }],
      total_matching: 1, next_cursor: null,
    } }) });
    const fields = await dirtyTraceForm(app);
    const activation = fields.trace.fire("click");
    await settle();
    const dialog = app.document.querySelector("dialog[open]");
    await app.byButton(discard ? "Discard edits" : "Keep editing", dialog).fire("click");
    await activation;
    const reads = app.requests.filter(request => request.route === "/provenance/entries");
    if (discard) {
      assert.equal(fields.target.isConnected, false);
      assert.equal(app.run("dirty"), false);
      assert.equal(app.byId("view-title").textContent, "Provenance");
      assert.equal(app.document.activeElement, app.byId("view-title"));
      assert.equal(reads.length, 1);
    } else {
      assert.equal(fields.target.isConnected, true);
      assert.equal(fields.target.value, "unsaved-report.html");
      assert.equal(app.run("dirty"), true);
      assert.equal(app.byId("view-title").textContent, "Trace & Reports");
      assert.equal(app.document.activeElement, fields.trace);
      assert.equal(reads.length, 0);
    }
  });
}


/** Create 53 wholly synthetic queue rows, enough to cross the 50-row display boundary. */
function queueRows() {
  return Array.from({ length: 53 }, (_, index) => ({
    item_id: `qi_synthetic${String(index + 1).padStart(4, "0")}`,
    control_id: `synthetic-${String(index + 1).padStart(2, "0")}`,
    reason_code: index < 3 ? "scope-decision-required" : "no-reviewed-mapping",
    summary: "Synthetic development fixture; no authentic review finding.",
  }));
}

/** Return a strong-version-bound API page with real filter and cursor semantics. */
function queuePage(url, version = "synthetic-version-1") {
  const all = queueRows();
  const reason = url.searchParams.get("reason_code");
  const matching = reason ? all.filter(row => row.reason_code === reason) : all;
  const size = Number(url.searchParams.get("page_size"));
  const offset = url.searchParams.get("cursor") === "synthetic-page-2" ? 50 : 0;
  return { resource_version: version, page: {
    items: matching.slice(offset, offset + size), total_matching: matching.length,
    next_cursor: offset + size < matching.length ? "synthetic-page-2" : null,
  } };
}

/** Find the committed pagination shell without relying on production private state. */
function pageParts(app) {
  const results = app.byId("view").querySelector("[data-page-results]");
  const status = app.byId("view").querySelector("[data-page-status]");
  const error = app.byId("view").querySelector("[data-page-error]");
  assert(results, "Missing persistent results region");
  assert(status, "Missing pagination status");
  assert(error, "Missing inline pagination error");
  return { results, status, error };
}

/** Open the actual Review Queue with synthetic responses and its real controls. */
async function queueApp(overrides = {}) {
  const app = harness({ "/review-queue/items": url => queuePage(url), ...overrides });
  await app.run('navigate("Review Queue")');
  return app;
}

test("forward/back pages and filter success focus the persistent result region", async () => {
  const app = await queueApp();
  const initial = pageParts(app);
  assert.equal(initial.status.getAttribute("role"), "status");
  assert.equal(initial.status.getAttribute("aria-live"), "polite");
  assert.match(initial.status.textContent, /53 matching items of 53 total.*Page 1/);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  const next = app.byButton("Next page");
  next.focus();
  await next.fire("click");
  assert.equal(pageParts(app).results, initial.results, "Results focus target must remain connected");
  assert.equal(app.document.activeElement, initial.results);
  assert.match(initial.results.textContent, /synthetic-51/);
  assert.doesNotMatch(initial.results.textContent, /synthetic-01/);
  assert.match(initial.status.textContent, /Page 2/);
  const previous = app.byButton("Previous page");
  previous.focus();
  await previous.fire("click");
  assert.equal(app.document.activeElement, initial.results);
  assert.match(initial.results.textContent, /synthetic-01/);
  assert.match(initial.status.textContent, /Page 1/);
  app.byLabel("Reason").value = "scope-decision-required";
  const apply = app.byButton("Apply filters");
  apply.focus();
  await apply.fire("click");
  assert.equal(app.document.activeElement, initial.results);
  assert.match(initial.status.textContent, /3 matching items of 53 total.*Page 1/);
  assert.match(initial.results.textContent, /synthetic-03/);
  assert.doesNotMatch(initial.results.textContent, /synthetic-04/);
  const unfiltered = app.requests.filter(request => request.route === "/review-queue/items"
    && new URL(request.url, "http://local").searchParams.get("page_size") === "1");
  assert(unfiltered.length > 0, "Filtered denominator must be independently requested at the same endpoint");
  assert(unfiltered.every(request => !new URL(request.url, "http://local").searchParams.has("reason_code")));
});

test("Overview open-review card focuses results while ordinary navigation focuses the heading", async () => {
  const app = harness({ "/review-queue/items": url => queuePage(url) });
  await app.run('navigate("Overview")');
  const card = app.byId("view").querySelectorAll("button").find(node => node.textContent.includes("Open review items"));
  assert(card);
  card.focus();
  await card.fire("click");
  assert.equal(app.document.activeElement, pageParts(app).results);
  await app.run('navigate("Overview")');
  await app.byButton("Review Queue", app.byId("navigation")).fire("click");
  assert.equal(app.document.activeElement, app.byId("view-title"));
});

test("empty filtered results retain a focusable region and exact zero-of-total count", async () => {
  const app = await queueApp();
  app.byLabel("Reason").value = "external-conflict";
  await app.byButton("Apply filters").fire("click");
  const parts = pageParts(app);
  assert.equal(app.document.activeElement, parts.results);
  assert.match(parts.results.textContent, /No matching items/);
  assert.match(parts.status.textContent, /0 matching items of 53 total.*Page 1/);
  assert.equal(app.byButton("Next page").hidden, true);
});

test("failed next-page fetch preserves rows/count/cursor and retries the exact staged page", async () => {
  let fail = true;
  const app = await queueApp({ "/review-queue/items": url => {
    if (url.searchParams.has("cursor") && fail) return { status: 500, body: {
      code: "internal-error", message: "Synthetic second-page failure.", retryable: true,
    } };
    return queuePage(url);
  } });
  const parts = pageParts(app);
  const rows = parts.results.textContent;
  const count = parts.status.textContent;
  await app.byButton("Next page").fire("click");
  assert.equal(parts.results.textContent, rows);
  assert(parts.status.textContent.includes(count), "Failed read must retain the previous exact count with its qualification");
  assert.match(parts.status.textContent, /Previous results/);
  assert.equal(app.document.activeElement, parts.error);
  assert.match(parts.error.textContent, /Synthetic second-page failure/);
  fail = false;
  await app.byButton("Retry page").fire("click");
  const cursorRequests = app.requests.filter(request => new URL(request.url, "http://local").searchParams.has("cursor"));
  assert.equal(cursorRequests.length, 2);
  assert(cursorRequests.every(request => new URL(request.url, "http://local").searchParams.get("cursor") === "synthetic-page-2"));
  assert.match(parts.status.textContent, /Page 2/);
  assert.equal(app.document.activeElement, parts.results);
  await app.byButton("Previous page").fire("click");
  assert.match(parts.status.textContent, /Page 1/);
});

test("filtered denominator from a different resource version cannot qualify or replace old rows", async () => {
  const app = await queueApp({ "/review-queue/items": url => {
    const version = url.searchParams.get("page_size") === "1" ? "synthetic-version-2" : "synthetic-version-1";
    return queuePage(url, version);
  } });
  const parts = pageParts(app);
  const rows = parts.results.textContent;
  const count = parts.status.textContent;
  app.byLabel("Reason").value = "scope-decision-required";
  await app.byButton("Apply filters").fire("click");
  assert.equal(parts.results.textContent, rows);
  assert(parts.status.textContent.includes(count), "Failed read must retain the previous exact count with its qualification");
  assert.match(parts.status.textContent, /Previous results/);
  assert.equal(parts.error.hidden, false);
  assert.equal(app.document.activeElement, parts.error);
  assert.match(parts.error.textContent, /changed|version|same snapshot|refresh/i);
});

test("version-conflict page failure preserves the first page and exposes exact API recovery", async () => {
  const app = await queueApp({ "/review-queue/items": url => url.searchParams.has("cursor")
    ? { status: 409, body: { code: "version-conflict", message: "The collection changed. Restart from its first page.",
      retryable: true, resource_version: "synthetic-version-2" } } : queuePage(url) });
  const parts = pageParts(app);
  const rows = parts.results.textContent;
  const count = parts.status.textContent;
  await app.byButton("Next page").fire("click");
  assert.equal(parts.results.textContent, rows);
  assert(parts.status.textContent.includes(count), "Failed read must retain the previous exact count with its qualification");
  assert.match(parts.status.textContent, /Previous results/);
  assert.equal(app.document.activeElement, parts.error);
  assert.match(parts.error.textContent, /version-conflict/);
  assert.match(parts.error.textContent, /synthetic-version-2/);
});

for (const fails of [false, true]) {
  test("late page " + (fails ? "failure" : "success") + " cannot replace or refocus a newer view", async () => {
    const response = deferred();
    const app = await queueApp({ "/review-queue/items": url => url.searchParams.has("cursor") ? response.promise : queuePage(url) });
    const parts = pageParts(app);
    const action = app.byButton("Next page").fire("click");
    await settle();
    const guarded = app.byButton("Next page");
    assert.equal(guarded.isConnected, true);
    assert.equal(guarded.disabled, false, "Busy guard must not use native disable and lose focus");
    assert.equal(guarded.getAttribute("aria-disabled"), "true");
    await app.run('navigate("Overview")');
    const status = app.byId("status").textContent;
    const focusCount = app.document.focusHistory.length;
    response.resolve(fails ? { status: 500, body: { code: "internal-error", message: "Obsolete page failure.", retryable: true } }
      : queuePage(new URL("http://local/?page_size=50&cursor=synthetic-page-2")));
    await action;
    assert.equal(parts.results.isConnected, false);
    assert.equal(app.byId("view-title").textContent, "Overview");
    assert.equal(app.byId("status").textContent, status);
    assert.equal(app.byId("error").hidden, true);
    assert.equal(app.document.activeElement, app.byId("view-title"));
    assert.equal(app.document.focusHistory.length, focusCount);
  });
}

test("initial paged-view failure installs an inline alert and preserves its focus after inert cleanup", async () => {
  const app = await queueApp({ "/review-queue/items": () => ({ status: 500,
    body: { code: "internal-error", message: "Initial queue unavailable.", retryable: true } }) });
  const parts = pageParts(app);
  assert.equal(parts.error.hidden, false);
  assert.equal(app.byId("view").inert, false);
  assert.equal(app.document.activeElement, parts.error);
  assert.match(parts.error.textContent, /Initial queue unavailable/);
  assert.doesNotMatch(parts.status.textContent, /0 matching items of 0 total/, "Failure must not become an authoritative empty inventory");
});

const operationId = "op_synthetic000001";
const operationRoute = "/operations/" + operationId;

/** Construct an OpenAPI-shaped synthetic operation; progress facts stay explicitly supplied. */
function operation(state, extra = {}) {
  return { operation_id: operationId, kind: "conversion", state,
    created_at: "2026-10-02T00:00:00Z", updated_at: "2026-10-02T00:00:01Z",
    cancel_requested: false, ...extra };
}

/** Provide a prepared receipt for UI rendering without claiming a real write or valid domain fixture. */
function proposedWrite() {
  return { preview_id: "prev_synthetic000001", operation_type: "policy-conversion", target: { status: "create", path: "synthetic-output.json" },
    semantic_summary: "Synthetic prepared conversion; no authoritative write.",
    validation: { state: "valid", error_count: 0, warning_count: 0, diagnostics: [] }, target_version: "synthetic-version-1", base_sha256: null,
    exact_bytes_sha256: "a".repeat(64), input_hashes: [], diff_text: "Synthetic diff", diff_truncated: false,
    receipt: { token: "synthetic-receipt-opaque-token", expires_at: "2026-10-02T01:00:00Z" } };
}

/** Supply a complete succeeded-conversion result with a synthetic prepared receipt. */
function conversionResult() {
  return { operation_id: operationId, products: [{ kind: "oscal-catalog", statement_count: 0 }],
    validation: { state: "valid", error_count: 0, warning_count: 0, diagnostics: [] }, preview: proposedWrite() };
}

/** Launch the actual conversion control; the fake transport substitutes API observations only. */
async function conversionApp(overrides = {}) {
  const app = harness({
    "/resources": () => ({ resource_version: "synthetic-version-1", page: { items: [{ resource_id: "res_synthetic000001", key: "synthetic-policy",
      role: "policy-source", path: "synthetic-policy.md", validation_state: "valid", sha256: "a".repeat(64), size_bytes: 1, stale: false, version: "synthetic-version-1" }], total_matching: 1, next_cursor: null } }),
    "/conversions": () => operation("pending"),
    [operationRoute]: () => operation("cancelled"),
    "/effects/previews/prev_synthetic000001": () => proposedWrite(),
    ...overrides,
  });
  app.run("readOnly = false;");
  await app.run('navigate("Policies & Artifacts")');
  app.byLabel("Policy source").value = "res_synthetic000001";
  app.byLabel("Output model").value = "oscal-catalog";
  app.byLabel("Output project-relative path").value = "synthetic-output.json";
  const prepare = app.byButton("Prepare conversion");
  prepare.focus();
  const activation = prepare.fire("click");
  await settle();
  return { ...app, prepare, activation };
}

/** Resolve the stable observed-operation row and its live status from actual rendered DOM. */
function operationParts(app) {
  const region = app.byId("operation-region");
  assert(region, "Missing persistent operation region");
  const row = region.querySelector('[data-operation-id="' + operationId + '"]');
  assert(row, "Missing observed operation ID row");
  const status = row.querySelector("[data-operation-status]");
  assert(status, "Missing stable operation live status");
  return { region, row, status };
}

test("supplied operation facts update stable live status once; absent facts stay indeterminate", async () => {
  let poll = 0;
  const responses = [operation("running", { progress: { completed_items: 2, total_items: 5 } }),
    operation("running", { progress: { completed_items: 2, total_items: 5 } }),
    operation("running", { progress: null }), operation("cancelled")];
  const app = await conversionApp({ [operationRoute]: () => responses[poll++] });
  const initial = operationParts(app);
  assert.equal(initial.status.getAttribute("role"), "status");
  assert.equal(initial.status.getAttribute("aria-live"), "polite");
  assert.equal(initial.status.getAttribute("aria-atomic"), "true");
  assert.match(initial.status.textContent, /pending/i);
  assert.match(initial.status.textContent, /indeterminate/i);
  const focusCount = app.document.focusHistory.length;
  await app.poll();
  assert.match(initial.status.textContent, /running/i);
  assert.match(initial.status.textContent, /Captured registrations: 2 of 5 reported/);
  assert.doesNotMatch(initial.status.textContent, /%|ETA|seconds|minutes/i);
  const writes = initial.status.textWrites.length;
  await app.poll();
  assert.equal(operationParts(app).status, initial.status);
  assert.equal(initial.status.textWrites.length, writes, "Repeated state/progress must not repeat the same live announcement");
  await app.poll();
  assert.match(initial.status.textContent, /indeterminate/i);
  assert.doesNotMatch(initial.status.textContent, /2 of 5/);
  await app.poll();
  await app.activation;
  assert.match(initial.status.textContent, /cancelled/i);
  assert.equal(app.document.focusHistory.length, focusCount, "Routine state changes must not steal focus");
  assert.equal(app.byId("preview-dialog").open, false);
});

test("cancellation keeps its connected focused button and acknowledgement survives an older poll", async () => {
  const cancellation = deferred();
  let poll = 0;
  const app = await conversionApp({
    [operationRoute]: () => ++poll === 1 ? operation("running") : operation("cancelled", { cancel_requested: true }),
    [operationRoute + "/cancellation"]: () => cancellation.promise,
  });
  const parts = operationParts(app);
  const cancel = app.byButton("Cancel pending operation", parts.row);
  cancel.focus();
  const activation = cancel.fire("click");
  await settle();
  assert.equal(cancel.isConnected, true);
  assert.equal(cancel.disabled, false);
  assert.equal(cancel.getAttribute("aria-disabled"), "true");
  assert.equal(app.document.activeElement, cancel);
  await cancel.fire("click");
  assert.equal(app.requests.filter(request => request.route.endsWith("/cancellation")).length, 1);
  cancellation.resolve(operation("running", { cancel_requested: true }));
  await activation;
  assert.match(parts.status.textContent, /Cancellation requested; awaiting terminal state/);
  await app.poll();
  assert.equal(app.byButton("Cancel pending operation", parts.row), cancel);
  assert.equal(cancel.isConnected, true);
  assert.match(parts.status.textContent, /Cancellation requested; awaiting terminal state/);
  assert.equal(cancel.getAttribute("aria-disabled"), "true");
  assert.equal(app.document.activeElement, cancel);
  await app.poll();
  await app.activation;
  assert.match(parts.status.textContent, /cancelled/i);
  assert.equal(app.byId("preview-dialog").open, false);
});

test("failed known-ID poll recovers by GET without repeating the preparation POST", async () => {
  let reads = 0;
  const app = await conversionApp({ [operationRoute]: () => ++reads === 1
    ? { status: 503, body: { code: "internal-error", message: "Synthetic polling interruption.", retryable: true } }
    : operation("cancelled") });
  const parts = operationParts(app);
  await app.poll();
  await app.activation;
  assert.equal(parts.row.isConnected, true);
  assert.match(parts.status.textContent, /unknown|unverified|could not|unavailable|interrupted|check/i);
  const recover = app.byButton("Check operation status", parts.row);
  recover.focus();
  await recover.fire("click");
  const preparations = app.requests.filter(request => request.route === "/conversions");
  assert.equal(preparations.length, 1);
  assert.equal(preparations[0].options.method, "POST");
  const polls = app.requests.filter(request => request.route === operationRoute);
  assert.equal(polls.length, 2);
  assert(polls.every(request => request.options.method === "GET"));
  assert.match(parts.status.textContent, /cancelled/i);
  assert.equal(app.byId("preview-dialog").open, false);
});

test("navigation during polling preserves its own status/focus and offers explicit prepared-write review", async () => {
  const app = await conversionApp({ [operationRoute]: () => operation("succeeded", { result: conversionResult() }) });
  const parts = operationParts(app);
  await app.run('navigate("Overview")');
  const status = app.byId("status").textContent;
  const focusCount = app.document.focusHistory.length;
  await app.poll();
  await app.activation;
  assert.equal(parts.row.isConnected, true);
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.match(parts.status.textContent, /succeeded/i);
  assert.doesNotMatch(parts.status.textContent, /saved|committed/i);
  const review = app.byButton("Review prepared write", parts.row);
  await review.fire("click");
  assert.equal(app.byId("preview-dialog").open, true);
  assert.equal(app.document.activeElement, app.byId("preview-title"));
  await app.byButton("Keep editing", app.byId("preview-dialog")).fire("click");
  await settle();
  assert.equal(app.byId("preview-dialog").open, false);
});

test("terminal preparation success reviews its receipt while a terminal failure never opens preview", async () => {
  const app = await conversionApp({ [operationRoute]: () => operation("succeeded", { result: conversionResult() }) });
  await app.poll();
  await app.activation;
  assert.equal(app.byId("preview-dialog").open, true);
  assert.equal(app.document.activeElement, app.byId("preview-title"));
  assert.match(operationParts(app).status.textContent, /succeeded/i);
  assert.doesNotMatch(operationParts(app).status.textContent, /saved|committed/i);
  assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
  await app.byButton("Keep editing", app.byId("preview-dialog")).fire("click");
  await settle();
  const failed = await conversionApp({ [operationRoute]: () => operation("failed", {
    error: { code: "validation-failed", message: "Synthetic terminal failure.", retryable: false },
  }) });
  await failed.poll();
  await failed.activation;
  assert.match(operationParts(failed).status.textContent, /failed/i);
  assert.match(operationParts(failed).status.textContent, /Synthetic terminal failure/);
  assert.equal(failed.byId("preview-dialog").open, false);
  assert.equal(failed.requests.some(request => request.route.startsWith("/effects/previews/")), false);
});

for (const invalid of [
  operation("unrecognized"),
  operation("running", { progress: { completed_items: -1, total_items: 5 } }),
  operation("running", { operation_id: "op_wrong000000001" }),
]) {
  test("unverified operation observation cannot become progress or a prepared-write success: " + JSON.stringify(invalid), async () => {
    const app = await conversionApp({ [operationRoute]: () => invalid });
    await app.poll();
    await app.activation;
    const parts = operationParts(app);
    assert.match(parts.status.textContent, /unverified|unknown|invalid|could not/i);
    assert.doesNotMatch(parts.status.textContent, /-1 of 5|succeeded|saved/i);
    assert.equal(app.byId("preview-dialog").open, false);
    assert(app.byButton("Check operation status", parts.row));
    assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
  });
}

test("shutdown suppresses a late successful poll, closes preview and stops all further requests", async () => {
  const response = deferred();
  const app = await conversionApp({ [operationRoute]: () => response.promise,
    "/session/shutdown": () => ({ state: "shutting-down" }) });
  const pendingPoll = app.poll();
  await settle();
  await app.byId("stop").fire("click");
  await app.byId("confirm-stop").fire("click");
  await settle();
  const count = app.requests.length;
  const status = app.byId("status").textContent;
  assert.equal(app.byId("workspace").hidden, true);
  assert.equal(app.byId("connection").textContent, "Stopped");
  assert.equal(app.document.activeElement, app.byId("main"));
  response.resolve(operation("succeeded", { result: conversionResult() }));
  await pendingPoll;
  await app.activation;
  await settle();
  assert.equal(app.requests.length, count);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.document.activeElement, app.byId("main"));
  assert.equal(app.run("capability"), "");
  const actions = app.byId("operation-region")?.querySelectorAll("button") ?? [];
  assert(actions.every(control => control.disabled || control.getAttribute("aria-disabled") === "true"),
    "Stopped rows must not expose activatable operation actions");
  for (const control of actions) await control.fire("click");
  assert.equal(app.requests.length, count);
  assert.equal(app.byId("status").textContent, status);
});


/** Test cancellation against an actually in-flight, older terminal poll response. */
test("cancellation invalidates an in-flight older poll before it can report success or open preview", async () => {
  const oldPoll = deferred();
  let reads = 0;
  const app = await conversionApp({
    [operationRoute]: () => ++reads === 1 ? oldPoll.promise : operation("cancelled", { cancel_requested: true }),
    [operationRoute + "/cancellation"]: () => operation("running", { cancel_requested: true }),
  });
  const parts = operationParts(app);
  await app.poll();
  const cancel = app.byButton("Cancel pending operation", parts.row);
  cancel.focus();
  await cancel.fire("click");
  assert.match(parts.status.textContent, /Cancellation requested; awaiting terminal state/);
  oldPoll.resolve(operation("succeeded", { result: conversionResult() }));
  await settle();
  assert.match(parts.status.textContent, /Cancellation requested; awaiting terminal state/);
  assert.doesNotMatch(parts.status.textContent, /succeeded/);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.document.activeElement, cancel);
  assert.equal(app.requests.some(request => request.route.startsWith("/effects/previews/")), false);
  await app.poll();
  await app.activation;
  assert.match(parts.status.textContent, /cancelled/i);
});

test("inconsistent supplied nonnegative progress stays indeterminate without an invented completion fraction", async () => {
  let reads = 0;
  const app = await conversionApp({ [operationRoute]: () => ++reads === 1
    ? operation("running", { progress: { completed_items: 6, total_items: 5 } }) : operation("cancelled") });
  await app.poll();
  const parts = operationParts(app);
  assert.match(parts.status.textContent, /running/i);
  assert.match(parts.status.textContent, /indeterminate|do not reconcile/i);
  assert.doesNotMatch(parts.status.textContent, /6 of 5|%|succeeded|saved/i);
  await app.poll();
  await app.activation;
});

test("shutdown while a prepared receipt is being read cannot open a late dialog or overwrite stopped focus", async () => {
  const receipt = deferred();
  const app = await conversionApp({
    [operationRoute]: () => operation("succeeded", { result: conversionResult() }),
    "/effects/previews/prev_synthetic000001": () => receipt.promise,
    "/session/shutdown": () => ({ state: "shutting-down" }),
  });
  await app.poll();
  assert.equal(app.requests.filter(request => request.route.startsWith("/effects/previews/")).length, 1);
  await app.byId("stop").fire("click");
  await app.byId("confirm-stop").fire("click");
  await settle();
  const requests = app.requests.length;
  const status = app.byId("status").textContent;
  const focusCount = app.document.focusHistory.length;
  receipt.resolve(proposedWrite());
  await app.activation;
  assert.equal(app.requests.length, requests);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.document.activeElement, app.byId("main"));
  assert.equal(app.document.focusHistory.length, focusCount);
});

test("polling read bound retains the exact known operation and resumes with GET only", async () => {
  let terminal = false;
  const app = await conversionApp({ [operationRoute]: () => operation(terminal ? "cancelled" : "running") });
  for (let read = 0; read < 120; read++) await app.poll();
  await app.activation;
  const parts = operationParts(app);
  assert.match(parts.status.textContent, /unknown/i);
  assert.match(parts.row.textContent, /read bound/i);
  assert.equal(app.requests.filter(request => request.route === operationRoute).length, 120);
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
  terminal = true;
  await app.byButton("Check operation status", parts.row).fire("click");
  assert.equal(app.requests.filter(request => request.route === operationRoute).length, 121);
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
  assert.match(parts.status.textContent, /cancelled/i);
});


test("supplied zero-of-zero progress remains reported facts without a completion percentage", async () => {
  let reads = 0;
  const app = await conversionApp({ [operationRoute]: () => ++reads === 1
    ? operation("running", { progress: { completed_items: 0, total_items: 0 } }) : operation("cancelled") });
  await app.poll();
  const parts = operationParts(app);
  assert.match(parts.status.textContent, /running/i);
  assert.match(parts.status.textContent, /Captured registrations: 0 of 0 reported/);
  assert.doesNotMatch(parts.status.textContent, /%|succeeded|saved|100/);
  await app.poll();
  await app.activation;
});

/** Exercise reported capture facts through the real conversion control and polling flow.
 * Synthetic observations prove UI units and generic counter compatibility, not a real capture. */
for (const progress of [
  { completed_items: 1, total_items: 101 },
  { completed_items: 101, total_items: 101 },
  { completed_items: 1001, total_items: 1001 },
]) {
  test("captured-registration facts preserve generic counters and remain preparation-only: " + JSON.stringify(progress), async () => {
    let reads = 0;
    const app = await conversionApp({ [operationRoute]: () => ++reads === 1
      ? operation("running", { progress }) : operation("cancelled") });
    const parts = operationParts(app);
    const focusCount = app.document.focusHistory.length;
    await app.poll();
    assert.equal(operationParts(app).row, parts.row);
    assert.equal(parts.status.textContent,
      `Operation running. Captured registrations: ${progress.completed_items} of ${progress.total_items} reported.`);
    assert.doesNotMatch(parts.status.textContent, /%|ETA|indeterminate|succeeded|saved|committed/i);
    assert.equal(app.byId("preview-dialog").open, false,
      "A complete capture denominator cannot open a prepared-write confirmation");
    assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
    assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
    assert.equal(app.document.focusHistory.length, focusCount);
    await app.poll();
    await app.activation;
    assert.match(parts.status.textContent, /cancelled/i);
    assert.doesNotMatch(parts.status.textContent, /Captured registrations|saved|committed/i);
    assert.equal(app.byId("preview-dialog").open, false);
  });
}

test("retrying a failed filter uses its captured criteria even after the form selection changes", async () => {
  let fail = true;
  const app = await queueApp({ "/review-queue/items": url => {
    if (fail && url.searchParams.get("reason_code") === "scope-decision-required") return { status: 500,
      body: { code: "internal-error", message: "Synthetic filtered read failure.", retryable: true } };
    return queuePage(url);
  } });
  const parts = pageParts(app);
  const prior = parts.results.textContent;
  app.byLabel("Reason").value = "scope-decision-required";
  await app.byButton("Apply filters").fire("click");
  assert.equal(parts.results.textContent, prior);
  assert.match(parts.status.textContent, /Previous results: 53 matching items of 53 total/);
  app.byLabel("Reason").value = "external-conflict";
  fail = false;
  await app.byButton("Retry page").fire("click");
  assert.match(parts.status.textContent, /3 matching items of 53 total/);
  const retried = app.requests.filter(request => new URL(request.url, "http://local").searchParams.has("reason_code")).at(-1);
  assert.equal(new URL(retried.url, "http://local").searchParams.get("reason_code"), "scope-decision-required");
  assert.equal(app.document.activeElement, parts.results);
});


for (const initialSuccess of [false, true]) {
  test("verified " + (initialSuccess ? "initial" : "polled") + " success survives receipt-read failure and retries the preview by GET", async () => {
    let previews = 0;
    const app = await conversionApp({
      "/conversions": () => initialSuccess ? operation("succeeded", { result: conversionResult() }) : operation("pending"),
      [operationRoute]: () => operation("succeeded", { result: conversionResult() }),
      "/effects/previews/prev_synthetic000001": () => ++previews === 1 ? { status: 503,
        body: { code: "internal-error", message: "Synthetic prepared-receipt read failure.", retryable: true } } : proposedWrite(),
    });
    if (!initialSuccess) await app.poll();
    await app.activation;
    const parts = operationParts(app);
    assert.match(parts.status.textContent, /Operation succeeded.*prepared write could not be loaded/i);
    assert.doesNotMatch(parts.status.textContent, /unknown|saved|committed/i);
    assert.match(parts.row.textContent, /Synthetic prepared-receipt read failure/);
    assert.equal(app.byButton("Check operation status", parts.row).hidden, true);
    assert.equal(app.byId("preview-dialog").open, false);
    await app.byButton("Review prepared write", parts.row).fire("click");
    assert.equal(app.byId("preview-dialog").open, true);
    assert.equal(app.document.activeElement, app.byId("preview-title"));
    assert.equal(parts.row.querySelector("[data-operation-error]").hidden, true,
      "A successful explicit review must clear its stale receipt-read error");
    assert.equal(parts.status.textContent, "Operation succeeded. Preparation is ready for review; no write has been confirmed.");
    assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
    assert.equal(app.requests.filter(request => request.route.startsWith("/effects/previews/")).length, 2);
    assert.equal(app.requests.filter(request => request.route === operationRoute).length, initialSuccess ? 0 : 1);
    assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
  });
}

test("malformed initial operation identity cannot offer a replay of the unverified preparation", async () => {
  const app = await conversionApp({ "/conversions": () => operation("pending", { operation_id: "op_bad!" }) });
  await app.activation;
  assert.equal(app.byId("error").hidden, false);
  assert.match(app.byId("error").textContent, /unsupported identity/i);
  assert.equal(app.byId("operation-region")?.querySelectorAll("[data-operation-id]").length ?? 0, 0);
  const retries = app.document.body.querySelectorAll("button").filter(control => control.textContent === "Retry the same request");
  assert(retries.every(control => control.hidden || control.disabled || control.getAttribute("aria-disabled") === "true"));
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
  assert.equal(app.requests.some(request => request.route.startsWith("/operations/")), false);
  assert.equal(app.byId("preview-dialog").open, false);
});


for (const terminalState of ["succeeded", "failed"]) {
  test("late cancellation failure retains the newer verified " + terminalState + " outcome", async () => {
    const cancellation = deferred();
    const terminal = terminalState === "succeeded"
      ? operation("succeeded", { result: conversionResult() })
      : operation("failed", { error: { code: "validation-failed", message: "Synthetic verified terminal failure.", retryable: false } });
    const app = await conversionApp({ [operationRoute]: () => terminal,
      [operationRoute + "/cancellation"]: () => cancellation.promise });
    const parts = operationParts(app);
    const cancel = app.byButton("Cancel pending operation", parts.row);
    cancel.focus();
    const attempt = cancel.fire("click");
    await settle();
    await app.poll();
    await app.activation;
    const verifiedStatus = parts.status.textContent;
    assert.match(verifiedStatus, new RegExp("Operation " + terminalState));
    cancellation.resolve({ status: 503, body: { code: "internal-error",
      message: "Synthetic delayed cancellation failure.", retryable: true } });
    await attempt;
    assert.equal(parts.status.textContent, verifiedStatus);
    assert.doesNotMatch(parts.status.textContent, /unknown/i);
    assert.match(parts.row.textContent, /Synthetic delayed cancellation failure/);
    assert.equal(app.byButton("Check operation status", parts.row).hidden, true);
    assert.equal(app.byButton("Cancel pending operation", parts.row).getAttribute("aria-disabled"), "true");
    assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
    assert.equal(app.requests.filter(request => request.route === operationRoute).length, 1);
    assert.equal(app.requests.filter(request => request.route.endsWith("/cancellation")).length, 1);
    assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
  });
}


/** Resolve the persistent unacknowledged preparation without reading its private key/state. */
function requestParts(app) {
  const region = app.byId("operation-region");
  assert(region, "Missing persistent request recovery region");
  const row = region.querySelector("[data-pending-request-id]");
  assert(row, "Missing unacknowledged preparation row");
  const status = row.querySelector("[data-request-status]");
  const error = row.querySelector("[data-request-error]");
  assert(status && error, "Missing request live status/error");
  return { region, row, status, error };
}

test("late initial POST failure preserves destination focus and recovers the exact request/key in a session row", async () => {
  const response = deferred();
  let sends = 0;
  const app = await conversionApp({ "/conversions": () => ++sends === 1 ? response.promise : operation("cancelled") });
  const original = app.requests.find(request => request.route === "/conversions");
  assert(original && original.options.headers["Idempotency-Key"]);
  await app.run('navigate("Overview")');
  const status = app.byId("status").textContent;
  const focusCount = app.document.focusHistory.length;
  response.resolve({ status: 503, body: { code: "internal-error", message: "Synthetic delayed preparation failure.", retryable: true } });
  await app.activation;
  const parts = requestParts(app);
  assert.equal(parts.row.hidden, false);
  assert.match(parts.status.textContent, /unknown|unacknowledged|could not/i);
  assert.match(parts.error.textContent, /Synthetic delayed preparation failure/);
  assert.equal(parts.status.getAttribute("role"), "status");
  assert.equal(app.byId("error").hidden, true);
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("preview-dialog").open, false);
  const retry = app.byButton("Retry the same request", parts.row);
  retry.focus();
  await retry.fire("click");
  const posts = app.requests.filter(request => request.route === "/conversions");
  assert.equal(posts.length, 2);
  assert.equal(posts[1].options.headers["Idempotency-Key"], original.options.headers["Idempotency-Key"]);
  assert.equal(posts[1].options.body, original.options.body);
  assert.equal(parts.row.isConnected, false, "Acknowledgement must transfer recovery to the exact observed operation ID");
  const acknowledged = operationParts(app);
  assert.match(acknowledged.status.textContent, /cancelled/i);
  assert.equal(app.document.activeElement, acknowledged.status,
    "Explicit Retry must transfer focus from its removed row to this exact operation");
  assert.equal(acknowledged.status.tabIndex, -1);
  assert(acknowledged.status.getAttribute("aria-label").includes(operationId));
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
});

test("explicit operation Review failure stays local and its successful retry clears only that stale receipt error", async () => {
  let reads = 0;
  const app = await conversionApp({ [operationRoute]: () => operation("succeeded", { result: conversionResult() }),
    "/effects/previews/prev_synthetic000001": () => ++reads === 1 ? { status: 503,
      body: { code: "internal-error", message: "Synthetic explicit review failure.", retryable: true } } : proposedWrite() });
  await app.run('navigate("Overview")');
  await app.poll();
  await app.activation;
  const parts = operationParts(app);
  const status = app.byId("status").textContent;
  const review = app.byButton("Review prepared write", parts.row);
  review.focus();
  await review.fire("click");
  assert.equal(review.isConnected, true);
  assert.equal(review.disabled, false);
  assert.equal(app.document.activeElement, review, "A failed read must retain the eligible initiating control");
  assert.equal(app.byId("error").hidden, true);
  assert.equal(app.byId("status").textContent, status);
  assert.match(parts.status.textContent, /Operation succeeded.*prepared write could not be loaded/i);
  assert.match(parts.row.querySelector("[data-operation-error]").textContent, /Synthetic explicit review failure/);
  assert.equal(app.byId("preview-dialog").open, false);
  await review.fire("click");
  assert.equal(app.byId("preview-dialog").open, true);
  assert.equal(app.document.activeElement, app.byId("preview-title"));
  assert.equal(parts.row.querySelector("[data-operation-error]").hidden, true);
  assert.equal(parts.status.textContent, "Operation succeeded. Preparation is ready for review; no write has been confirmed.");
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
  assert.equal(app.requests.filter(request => request.route.startsWith("/effects/previews/")).length, 2);
});

test("late explicit operation Review error cannot steal a newer view's focus or announce a global failure", async () => {
  const receipt = deferred();
  const app = await conversionApp({ [operationRoute]: () => operation("succeeded", { result: conversionResult() }),
    "/effects/previews/prev_synthetic000001": () => receipt.promise });
  await app.run('navigate("Overview")');
  await app.poll();
  await app.activation;
  const parts = operationParts(app);
  const review = app.byButton("Review prepared write", parts.row);
  review.focus();
  const read = review.fire("click");
  await settle();
  assert.equal(review.disabled, false);
  await app.run('navigate("Trace & Reports")');
  const status = app.byId("status").textContent;
  const focusCount = app.document.focusHistory.length;
  receipt.resolve({ status: 503, body: { code: "internal-error", message: "Synthetic obsolete explicit review failure.", retryable: true } });
  await read;
  assert.equal(app.byId("view-title").textContent, "Trace & Reports");
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.byId("error").hidden, true);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.match(parts.status.textContent, /succeeded/i);
});


test("late direct registration preview offers local Review without replay or an unsolicited modal", async () => {
  const response = deferred();
  const preview = { ...proposedWrite(), operation_type: "workspace-index-update",
    target: { status: "create", path: "forge.workspace.json" }, semantic_summary: "Synthetic registration preparation." };
  const app = harness({ "/resources/register": () => response.promise,
    "/effects/previews/prev_synthetic000001": () => preview });
  app.run("readOnly = false;");
  await app.run('navigate("Policies & Artifacts")');
  app.byLabel("Resource role").value = "policy-source";
  app.byLabel("Project-relative file path").value = "synthetic-policy.md";
  app.byLabel("Stable resource key").value = "synthetic-policy";
  const preparation = app.byButton("Preview registration").fire("click");
  await settle();
  await app.run('navigate("Overview")');
  const status = app.byId("status").textContent;
  const focusCount = app.document.focusHistory.length;
  response.resolve({ validation: { state: "valid", error_count: 0, warning_count: 0, diagnostics: [] }, preview });
  await preparation;
  const parts = requestParts(app);
  assert.match(parts.status.textContent, /Preparation is ready for review/);
  assert.equal(app.byButton("Retry the same request", parts.row).hidden, true);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.byId("error").hidden, true);
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  const review = app.byButton("Review prepared write", parts.row);
  await review.fire("click");
  assert.equal(app.byId("preview-dialog").open, true);
  assert.equal(app.document.activeElement, app.byId("preview-title"));
  assert.equal(app.requests.filter(request => request.route === "/resources/register").length, 1);
  assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
});

test("global and persistent Retry share one in-flight replay guard and the original idempotency key", async () => {
  const replay = deferred();
  let sends = 0;
  const app = await conversionApp({ "/conversions": () => ++sends === 1 ? { status: 503,
    body: { code: "internal-error", message: "Synthetic initial response loss.", retryable: true } } : replay.promise });
  await app.activation;
  const parts = requestParts(app);
  const globalRetry = app.byButton("Retry the same request", app.byId("error"));
  const sessionRetry = app.byButton("Retry the same request", parts.row);
  const retry = globalRetry.fire("click");
  await settle();
  await sessionRetry.fire("click");
  const inFlight = app.requests.filter(request => request.route === "/conversions");
  assert.equal(inFlight.length, 2, "A second recovery control must not start a concurrent preparation POST");
  assert.equal(inFlight[0].options.headers["Idempotency-Key"], inFlight[1].options.headers["Idempotency-Key"]);
  assert.equal(inFlight[0].options.body, inFlight[1].options.body);
  replay.resolve(operation("cancelled"));
  await retry;
  assert.equal(parts.row.isConnected, false);
  assert.match(operationParts(app).status.textContent, /cancelled/);
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 2);
});


test("old poll rejection after cancellation acknowledgement cannot erase sticky state or stop fresh polling", async () => {
  const obsoleteRead = deferred();
  let reads = 0;
  const app = await conversionApp({
    [operationRoute]: () => ++reads === 1 ? obsoleteRead.promise : operation("cancelled", { cancel_requested: true }),
    [operationRoute + "/cancellation"]: () => operation("running", { cancel_requested: true }),
  });
  const parts = operationParts(app);
  await app.poll();
  const cancel = app.byButton("Cancel pending operation", parts.row);
  cancel.focus();
  await cancel.fire("click");
  const acknowledged = parts.status.textContent;
  assert.match(acknowledged, /Cancellation requested; awaiting terminal state/);
  obsoleteRead.resolve({ status: 503, body: { code: "internal-error", message: "Synthetic obsolete polling failure.", retryable: true } });
  await settle();
  assert.equal(parts.status.textContent, acknowledged);
  assert.doesNotMatch(parts.status.textContent, /unknown/i);
  assert.equal(parts.row.querySelector("[data-operation-error]").hidden, true);
  assert.equal(app.document.activeElement, cancel);
  await app.poll();
  await app.activation;
  assert.match(parts.status.textContent, /cancelled/i);
  assert.equal(app.requests.filter(request => request.route === operationRoute).length, 2);
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 1);
});

test("a background initial POST acknowledgement does not focus its new operation row over a newer destination", async () => {
  const response = deferred();
  const app = await conversionApp({ "/conversions": () => response.promise });
  await app.run('navigate("Overview")');
  const status = app.byId("status").textContent;
  const focusCount = app.document.focusHistory.length;
  response.resolve(operation("pending"));
  await settle();
  const parts = operationParts(app);
  assert.equal(parts.row.isConnected, true);
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.byId("status").textContent, status);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("operation-region").querySelectorAll("[data-pending-request-id]").length, 0);
  await app.poll();
  await app.activation;
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.document.focusHistory.length, focusCount);
  assert.equal(app.byId("preview-dialog").open, false);
});


test("request display bound prevents the 257th new preparation POST before transport", async () => {
  const app = await conversionApp({ "/conversions": () => ({ status: 503,
    body: { code: "internal-error", message: "Synthetic unacknowledged preparation.", retryable: true } }) });
  await app.activation;
  for (let attempt = 1; attempt < 256; attempt++) await app.prepare.fire("click");
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 256);
  assert.equal(app.byId("operation-region").querySelectorAll("[data-pending-request-id]").length, 256);
  await app.prepare.fire("click");
  assert.equal(app.requests.filter(request => request.route === "/conversions").length, 256);
  assert.equal(app.byId("operation-region").querySelectorAll("[data-pending-request-id]").length, 256);
  assert.match(app.byId("error").textContent, /request display bound/i);
});

test("shutdown during pending request Retry suppresses a late acknowledgement and leaves no actionable recovery", async () => {
  const replay = deferred();
  let sends = 0;
  const app = await conversionApp({ "/conversions": () => ++sends === 1 ? { status: 503,
    body: { code: "internal-error", message: "Synthetic initial response loss.", retryable: true } } : replay.promise,
    "/session/shutdown": () => ({ state: "shutting-down" }) });
  await app.activation;
  const parts = requestParts(app);
  const retry = app.byButton("Retry the same request", parts.row).fire("click");
  await settle();
  await app.byId("stop").fire("click");
  await app.byId("confirm-stop").fire("click");
  await settle();
  const count = app.requests.length;
  const status = app.byId("status").textContent;
  const rowStatus = parts.status.textContent;
  replay.resolve(operation("pending"));
  await retry;
  assert.equal(app.requests.length, count);
  assert.equal(app.byId("operation-region").querySelectorAll("[data-operation-id]").length, 0);
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.byId("status").textContent, status);
  assert.equal(parts.status.textContent, rowStatus);
  assert.equal(app.document.activeElement, app.byId("main"));
  assert(parts.row.querySelectorAll("button").every(control => control.getAttribute("aria-disabled") === "true" || control.disabled));
});

test("background direct-preview Retry transfers its hidden invoker focus to the named prepared status", async () => {
  const failure = deferred();
  let sends = 0;
  const preview = { ...proposedWrite(), operation_type: "workspace-index-update",
    target: { status: "create", path: "forge.workspace.json" }, semantic_summary: "Synthetic registration preparation." };
  const app = harness({ "/resources/register": () => ++sends === 1 ? failure.promise
    : { validation: { state: "valid", error_count: 0, warning_count: 0, diagnostics: [] }, preview },
    "/effects/previews/prev_synthetic000001": () => preview });
  app.run("readOnly = false;");
  await app.run('navigate("Policies & Artifacts")');
  app.byLabel("Resource role").value = "policy-source";
  app.byLabel("Project-relative file path").value = "synthetic-policy.md";
  app.byLabel("Stable resource key").value = "synthetic-policy";
  const initial = app.byButton("Preview registration").fire("click");
  await settle();
  await app.run('navigate("Overview")');
  failure.resolve({ status: 503, body: { code: "internal-error", message: "Synthetic delayed registration failure.", retryable: true } });
  await initial;
  const parts = requestParts(app);
  const retry = app.byButton("Retry the same request", parts.row);
  retry.focus();
  await retry.fire("click");
  assert.equal(retry.hidden, true);
  assert.equal(app.document.activeElement, parts.status,
    "Hiding the selected Retry must preserve focus on this exact prepared request");
  assert.equal(parts.status.tabIndex, -1);
  assert(parts.status.getAttribute("aria-label"));
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.requests.filter(request => request.route === "/resources/register").length, 2);
  assert.equal(app.byButton("Review prepared write", parts.row).hidden, false);
});

for (const fails of [false, true]) {
  test("shutdown suppresses an initial paged render's late " + (fails ? "failure" : "success"), async () => {
    const response = deferred();
    const app = harness({ "/review-queue/items": () => response.promise,
      "/session/shutdown": () => ({ state: "shutting-down" }) });
    const loading = app.run('navigate("Review Queue")');
    await settle();
    await app.byId("stop").fire("click");
    await app.byId("confirm-stop").fire("click");
    await settle();
    const title = app.byId("view-title").textContent;
    const view = app.byId("view").textContent;
    const status = app.byId("status").textContent;
    const focusCount = app.document.focusHistory.length;
    response.resolve(fails ? { status: 503, body: { code: "internal-error", message: "Synthetic stopped list failure.", retryable: true } }
      : queuePage(new URL("http://local/?page_size=50")));
    await loading;
    assert.equal(app.byId("view-title").textContent, title);
    assert.equal(app.byId("view").textContent, view);
    assert.equal(app.byId("status").textContent, status);
    assert.match(status, /Workspace stopped/);
    assert.equal(app.byId("error").hidden, true);
    assert.equal(app.document.activeElement, app.byId("main"));
    assert.equal(app.document.focusHistory.length, focusCount);
  });
}


/** Return a closed-contract synthetic unlock response without persisting a real credential. */
function unlockedSession(readOnly = true) {
  return { capability: "synthetic-test-capability-32-characters", session: {
    session_id: "sess_synthetic001", mode: "browser", api_major: 1, read_only: readOnly,
    contract_version: "1.0.0", project_label: "Synthetic project",
  } };
}

/** Observe actual form submission; this fake DOM does not synthesize native Enter defaults. */
function beginUnlock(app, trigger = "field") {
  const field = app.byId("passphrase");
  const form = app.byId("unlock-form");
  const submit = form.querySelector("button");
  field.value = "synthetic test passphrase";
  (trigger === "field" ? field : submit).focus();
  return { field, form, submit, action: form.fire("submit") };
}

const throttleMessage = "Too many attempts — retry available in 2 seconds. Wait, then retry. If repeated, stop and relaunch the workspace from the terminal.";
const throttledReply = { status: 429, body: { code: "unlock-throttled", message: throttleMessage, retryable: true } };

test("locked source initialization focuses and describes the passphrase field", () => {
  const app = harness();
  assert.equal(app.document.activeElement, app.byId("passphrase"));
  assert.equal(app.byId("status").textContent, "Workspace locked — passphrase required.");
  assert.equal(app.byId("passphrase").getAttribute("aria-describedby"), "status");
  assert.equal(app.requests.length, 0);
});

for (const trigger of ["field", "submit"]) {
  test("pending unlock retains the " + trigger + " trigger and rejects repeated submission", async () => {
    const reply = deferred();
    const app = harness({ "/session/unlock": () => reply.promise });
    const attempt = beginUnlock(app, trigger);
    await settle();
    assert.equal(app.document.activeElement, trigger === "field" ? attempt.field : attempt.submit);
    assert.equal(attempt.form.getAttribute("aria-busy"), "true");
    assert.equal(attempt.submit.getAttribute("aria-disabled"), "true");
    assert.equal(attempt.submit.disabled, false, "Native disabling must not drop the pending trigger's focus");
    assert.equal(app.byId("status").textContent, "Unlocking workspace…");
    await attempt.form.fire("submit");
    assert.equal(app.requests.length, 1);
    assert.equal(app.requests[0].options.method, "POST");
    assert.deepEqual(JSON.parse(app.requests[0].options.body), { passphrase: "synthetic test passphrase" });
    reply.resolve(unlockedSession());
    await attempt.action;
    assert.equal(attempt.field.value, "");
    assert.equal(attempt.form.getAttribute("aria-busy"), "false");
    assert.equal(attempt.submit.getAttribute("aria-disabled"), "false");
    assert.equal(app.document.activeElement, app.byId("main"));
    assert.equal(app.byId("status").textContent, "Project state loaded.");
  });

  test("typed throttle preserves the " + trigger + " owned field retry target and exact server message", async () => {
    const reply = deferred();
    const app = harness({ "/session/unlock": () => reply.promise });
    const attempt = beginUnlock(app, trigger);
    await settle();
    reply.resolve(throttledReply);
    await attempt.action;
    assert.equal(app.document.activeElement, attempt.field);
    assert.equal(app.byId("status").textContent, throttleMessage);
    assert.equal(app.byId("error").hidden, true);
    assert.equal(attempt.field.value, "", "Completed failures retain credential clearing");
    assert.equal(attempt.form.getAttribute("aria-busy"), "false");
    assert.equal(attempt.submit.getAttribute("aria-disabled"), "false");
    assert.equal(app.run("capability"), "");
    assert.equal(app.requests.filter(request => request.route === "/project/summary").length, 0);
    app.routes["/session/unlock"] = () => unlockedSession();
    const retry = beginUnlock(app);
    await retry.action;
    assert.equal(app.document.activeElement, app.byId("main"));
    assert.equal(app.requests.filter(request => request.route === "/session/unlock").length, 2);
  });
}

test("a late throttle cannot take focus from a newer connected control", async () => {
  const reply = deferred();
  const app = harness({ "/session/unlock": () => reply.promise });
  const attempt = beginUnlock(app);
  await settle();
  const newer = app.byId("refresh"); newer.focus();
  reply.resolve(throttledReply);
  await attempt.action;
  assert.equal(app.document.activeElement, newer);
  assert.equal(app.byId("status").textContent, throttleMessage);
  assert.equal(app.byId("error").hidden, true);
});

test("ordinary failed unlock clears the credential and focuses the safe summary", async () => {
  const app = harness({ "/session/unlock": () => ({ status: 401, body: {
    code: "unlock-failed", message: "The passphrase did not unlock this session.", retryable: true,
  } }) });
  const attempt = beginUnlock(app); await attempt.action;
  assert.equal(attempt.field.value, "");
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.match(app.byId("error").textContent, /unlock-failed.*did not unlock.*Retryable: true/);
  assert.equal(app.byId("status").textContent, "Workspace locked — passphrase required.");
  assert.equal(app.run("capability"), "");
});

test("invalid unlock length is rejected before transport and releases the pending guard", async () => {
  const app = harness();
  const field = app.byId("passphrase"); field.value = "short";
  await app.byId("unlock-form").fire("submit");
  assert.equal(app.requests.length, 0);
  assert.equal(field.value, "");
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.match(app.byId("error").textContent, /15–128/);
  assert.equal(app.run("unlockPending"), false);
});

/** Provide the documented terminal commit result; these bytes are a synthetic transport fixture. */
function committedWrite(target = "synthetic-output.json") {
  return operation("succeeded", { kind: "commit", result: { write_committed: true,
    committed_sha256: "a".repeat(64), target_path: target, new_version: "synthetic-version-2" } });
}

/** Render the actual preview and commit controls, using only synthetic API observations. */
async function commitApp(overrides = {}, exported = false) {
  const app = harness({
    "/effects/previews/prev_synthetic000001": () => proposedWrite(),
    "/effects/commits": () => operation("pending", { kind: "commit" }),
    [operationRoute]: () => committedWrite(),
    ...overrides,
  });
  app.run("readOnly = false;");
  await app.run('navigate(' + JSON.stringify(exported ? "Trace & Reports" : "Policies & Artifacts") + ')');
  const invoker = app.byButton(exported ? "Prepare export" : "Preview registration");
  invoker.focus();
  await app.run('preview(' + JSON.stringify(proposedWrite()) + (exported ? ',' + JSON.stringify(operationId) : '') + ')');
  app.run("dirty = true;");
  return { ...app, invoker, confirm: app.byButton("Confirm this exact write", app.byId("preview-dialog")) };
}

/** Assert the actual receipt confirmation/key and that UI recovery never resends a commit. */
function oneExactCommit(app) {
  const requests = app.requests.filter(request => request.route === "/effects/commits");
  assert.equal(requests.length, 1);
  assert.equal(requests[0].options.method, "POST");
  assert.deepEqual(JSON.parse(requests[0].options.body), { receipt: "synthetic-receipt-opaque-token", observed_version: "synthetic-version-1", confirmed: true });
  assert.match(requests[0].options.headers["Idempotency-Key"], /^[a-f0-9-]{36}$/);
}

for (const exported of [false, true]) {
  test("verified " + (exported ? "export" : "registration") + " focuses the refreshed heading after queued native close", async () => {
    const app = await commitApp({}, exported);
    app.document.holdCloseEvents = true;
    app.confirm.focus();
    const action = app.confirm.fire("click"); await settle();
    assert.equal(app.byId("preview-dialog").open, false);
    assert.equal(app.invoker.isConnected, false, "Refreshed content replaces the dialog's original return target");
    assert.equal(app.document.activeElement, app.document.body);
    assert.equal(app.document.closeEvents.length, 1);
    assert.doesNotMatch(app.byId("status").textContent, /^Saved /, "Saved focus publication must await native close");
    app.document.closeEvents.shift()();
    await action;
    assert.equal(app.document.activeElement, app.byId("view-title"));
    assert.equal(app.byId("status").textContent, "Saved synthetic-output.json.");
    assert.equal(app.run("dirty"), false);
    assert.equal(app.byId("view").inert, false);
    if (exported) assert.equal(app.byButton("Download committed redacted report").isConnected, true);
    else assert.equal(app.byId("view").querySelectorAll("button").some(node => node.textContent === "Download committed redacted report"), false);
    oneExactCommit(app);
  });
}

for (const inline of [false, true]) {
  test("verified write with " + (inline ? "inline table" : "generic") + " refresh failure focuses saved-but-refresh-failed summary after close", async () => {
    const app = await commitApp();
    if (inline) {
      app.run('activeView = "Review Queue";');
      app.routes["/review-queue/items"] = () => ({ status: 500, body: { code: "internal-error", message: "Synthetic saved queue failure.", retryable: true } });
    } else app.routes["/resources"] = () => ({ status: 500, body: { code: "internal-error", message: "Synthetic saved resources failure.", retryable: true } });
    app.document.holdCloseEvents = true;
    const action = app.confirm.fire("click"); await settle();
    assert.equal(app.byId("preview-dialog").open, false);
    assert.equal(app.document.closeEvents.length, 1);
    app.document.closeEvents.shift()(); await action;
    assert.equal(app.document.activeElement, app.byId("error"));
    assert.match(app.byId("error").textContent, /write was saved, but the view could not be refreshed.*Synthetic saved/);
    assert.equal(app.byId("status").textContent, "Saved synthetic-output.json.");
    assert.equal(app.run("dirty"), false);
    oneExactCommit(app);
  });
}

test("rejected commit retains its preview and unsaved state with a focused modal error", async () => {
  const app = await commitApp({ "/effects/commits": () => ({ status: 409, body: {
    code: "receipt-expired", message: "Prepare a new preview.", retryable: false,
  } }) });
  await app.confirm.fire("click");
  const dialog = app.byId("preview-dialog"); const error = dialog.querySelector("[role=alert]");
  assert.equal(dialog.open, true);
  assert.equal(app.document.activeElement, error);
  assert.match(error.textContent, /receipt-expired.*Prepare a new preview/);
  assert.equal(app.run("dirty"), true);
  assert.equal(app.requests.filter(request => request.route === operationRoute).length, 0);
  oneExactCommit(app);
});

for (const interruption of ["Escape", "navigation", "shutdown"]) {
  test("late confirmed-write observation after " + interruption + " cannot publish Saved or steal newer focus", async () => {
    const result = deferred();
    const app = await commitApp({ [operationRoute]: () => result.promise,
      "/session/shutdown": () => ({ state: "shutting-down" }) });
    const action = app.confirm.fire("click"); await settle();
    app.byId("preview-dialog").escape(); await settle();
    if (interruption === "navigation") { app.run("dirty = false;"); await app.run('navigate("Overview")'); }
    else if (interruption === "shutdown") { await app.byId("stop").fire("click"); await app.byId("confirm-stop").fire("click"); }
    else app.byId("refresh").focus();
    const destination = app.document.activeElement;
    const status = app.byId("status").textContent;
    result.resolve(committedWrite()); await action; await settle();
    assert.equal(app.document.activeElement, destination);
    assert.equal(app.byId("status").textContent, status);
    assert.equal(app.byId("preview-dialog").open, false);
    assert.equal(app.run("dirty"), interruption === "navigation" ? false : true);
    oneExactCommit(app);
  });
}

test("Escape while a verified commit refresh is pending suppresses later render and saved focus", async () => {
  const app = await commitApp(); const refresh = deferred();
  app.routes["/resources"] = () => refresh.promise;
  const action = app.confirm.fire("click"); await settle();
  app.byId("preview-dialog").escape(); await settle();
  app.byId("main").focus();
  const view = app.byId("view").children[0];
  refresh.resolve({ resource_version: "synthetic-version-1", page: { items: [], total_matching: 0, next_cursor: null } });
  await action;
  assert.equal(app.byId("view").children[0], view);
  assert.equal(app.document.activeElement, app.byId("main"));
  assert.doesNotMatch(app.byId("status").textContent, /^Saved /);
  assert.equal(app.run("dirty"), true);
  assert.equal(app.byId("view").inert, false);
  oneExactCommit(app);
});


for (const dismissal of ["Escape", "Keep editing"]) {
  test("queued " + dismissal + " close cannot invalidate a successor preview or publish the old commit", async () => {
    const oldResult = deferred(); let reads = 0;
    const second = { ...proposedWrite(), preview_id: "prev_synthetic000002",
      receipt: { token: "synthetic-successor-receipt-token", expires_at: "2026-10-02T01:00:00Z" } };
    const app = await commitApp({
      [operationRoute]: () => ++reads === 1 ? oldResult.promise : committedWrite(),
      "/effects/previews/prev_synthetic000002": () => second,
    });
    const oldAction = app.confirm.fire("click"); await settle();
    app.document.holdCloseEvents = true;
    let dismissed;
    if (dismissal === "Escape") app.byId("preview-dialog").escape();
    else dismissed = app.byButton("Keep editing", app.byId("preview-dialog")).fire("click");
    const successor = app.run('preview(' + JSON.stringify(second) + ')');
    await settle();
    assert.equal(app.byId("preview-dialog").open, false, "Successor must wait for old native close lifecycle");
    assert.equal(app.document.closeEvents.length, 1);
    app.document.closeEvents.shift()();
    if (dismissed) await dismissed;
    await successor;
    const newTitle = app.byId("preview-title");
    assert.equal(app.byId("preview-dialog").open, true);
    assert.equal(app.document.activeElement, newTitle);
    oldResult.resolve(committedWrite()); await oldAction;
    assert.equal(app.byId("preview-dialog").open, true);
    assert.equal(app.document.activeElement, newTitle);
    assert.equal(app.run("dirty"), true);
    assert.doesNotMatch(app.byId("status").textContent, /^Saved /);
    app.document.holdCloseEvents = false;
    await app.byButton("Confirm this exact write", app.byId("preview-dialog")).fire("click");
    assert.equal(app.byId("preview-dialog").open, false);
    assert.equal(app.document.activeElement, app.byId("view-title"));
    const commits = app.requests.filter(request => request.route === "/effects/commits");
    assert.equal(commits.length, 2, "Each explicitly confirmed receipt is sent once; no recovery replay");
    assert.deepEqual(commits.map(request => JSON.parse(request.options.body).receipt), ["synthetic-receipt-opaque-token", "synthetic-successor-receipt-token"]);
    assert.notEqual(commits[0].options.headers["Idempotency-Key"], commits[1].options.headers["Idempotency-Key"]);
  });
}

test("ineligible late direct preparation cannot revoke the visible receipt's confirmation", async () => {
  const direct = deferred();
  const app = harness({ "/resources/register": () => direct.promise,
    "/effects/previews/prev_synthetic000001": () => proposedWrite(),
    "/effects/commits": () => operation("pending", { kind: "commit" }),
    [operationRoute]: () => committedWrite(),
  });
  app.run("readOnly = false;"); await app.run('navigate("Policies & Artifacts")');
  const background = app.run('effect("/resources/register", "POST", {role:"policy-source",path:"synthetic-policy.md",key:"fixture"})');
  await settle();
  await app.run('preview(' + JSON.stringify(proposedWrite()) + ')');
  const title = app.byId("preview-title");
  const reads = app.requests.filter(request => request.route.startsWith("/effects/previews/")).length;
  direct.resolve({ preview: proposedWrite() }); await background;
  assert.equal(app.byId("preview-dialog").open, true);
  assert.equal(app.document.activeElement, title);
  assert.equal(app.requests.filter(request => request.route.startsWith("/effects/previews/")).length, reads,
    "Already-ineligible background preview must not fetch or reserve current preview ownership");
  assert.equal(app.requests.filter(request => request.route === "/resources/register").length, 1);
  await app.byButton("Confirm this exact write", app.byId("preview-dialog")).fire("click");
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.byId("status").textContent, "Saved synthetic-output.json.");
  oneExactCommit(app);
});

test("navigation during the queued confirmed-close event retains its newer heading and status", async () => {
  const app = await commitApp(); app.document.holdCloseEvents = true;
  const action = app.confirm.fire("click"); await settle();
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.document.closeEvents.length, 1);
  await app.run('navigate("Overview")');
  const status = app.byId("status").textContent;
  app.document.closeEvents.shift()(); await action;
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.byId("view-title").textContent, "Overview");
  assert.equal(app.byId("status").textContent, status);
  assert.doesNotMatch(status, /^Saved /);
  oneExactCommit(app);
});


for (const fails of [false, true]) {
  test("dismissed confirmed-refresh " + (fails ? "failure" : "success") + " preserves usable old pagination and filters", async () => {
    const app = await queueApp({
      "/effects/previews/prev_synthetic000001": () => proposedWrite(),
      "/effects/commits": () => operation("pending", { kind: "commit" }),
      [operationRoute]: () => committedWrite(),
    });
    const retained = pageParts(app); const rows = retained.results.textContent;
    const next = app.byButton("Next page");
    await app.run('preview(' + JSON.stringify(proposedWrite()) + ')');
    app.run("dirty = true;");
    const refresh = deferred(); app.routes["/review-queue/items"] = () => refresh.promise;
    const action = app.byButton("Confirm this exact write", app.byId("preview-dialog")).fire("click");
    await settle(); app.byId("preview-dialog").escape(); await settle();
    app.byId("main").focus();
    refresh.resolve(fails ? { status: 500, body: { code: "internal-error", message: "Obsolete confirmed-refresh failure.", retryable: true } }
      : queuePage(new URL("http://local/?page_size=50")));
    await action;
    assert.equal(retained.results.isConnected, true);
    assert.equal(retained.results.textContent, rows);
    assert.equal(app.document.activeElement, app.byId("main"));
    assert.equal(app.byId("view").inert, false);
    assert.equal(app.byId("error").hidden, true);
    assert.doesNotMatch(app.byId("status").textContent, /^Saved /);
    app.routes["/review-queue/items"] = url => queuePage(url);
    const reads = app.requests.filter(request => request.route === "/review-queue/items").length;
    next.focus(); await next.fire("click");
    assert.equal(app.requests.filter(request => request.route === "/review-queue/items").length, reads + 1,
      "Retained Next handler must still own the visible page after discarded staging");
    assert.match(retained.status.textContent, /Page 2/);
    assert.equal(app.document.activeElement, retained.results);
    app.byLabel("Reason").value = "scope-decision-required";
    const apply = app.byButton("Apply filters"); apply.focus(); await apply.fire("click");
    assert.equal(retained.status.textContent, "3 matching items of 53 total · Page 1.");
    assert.equal(app.document.activeElement, retained.results);
    oneExactCommit(app);
  });
}


test("unsupported API major clears unlock data and focuses the compatibility error", async () => {
  const app = harness({ "/session/unlock": () => {
    const response = unlockedSession(); response.session.api_major = 2; return response;
  } });
  const attempt = beginUnlock(app); await attempt.action;
  assert.equal(attempt.field.value, "");
  assert.equal(app.run("capability"), "");
  assert.equal(app.run("unlockPending"), false);
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.match(app.byId("error").textContent, /requires API version 1.*matching workspace assets/);
  assert.equal(app.requests.filter(request => request.route === "/project/summary").length, 0);
});

test("shutdown during queued confirmed close suppresses Saved and preserves stopped focus", async () => {
  const app = await commitApp({ "/session/shutdown": () => ({ state: "shutting-down" }) });
  app.document.holdCloseEvents = true;
  const action = app.confirm.fire("click"); await settle();
  assert.equal(app.byId("preview-dialog").open, false);
  assert.equal(app.document.closeEvents.length, 1);
  await app.byId("stop").fire("click"); await app.byId("confirm-stop").fire("click");
  const stopped = app.byId("status").textContent;
  while (app.document.closeEvents.length) app.document.closeEvents.shift()();
  await action;
  assert.equal(app.document.activeElement, app.byId("main"));
  assert.equal(app.byId("status").textContent, stopped);
  assert.match(stopped, /Workspace stopped/);
  assert.equal(app.run("capability"), "");
  oneExactCommit(app);
});

test("committed export download failure uses its authenticated route and cannot replay the write", async () => {
  const app = await commitApp({ ["/exports/" + operationId + "/download"]: () => ({ status: 409,
    body: { code: "version-conflict", message: "Synthetic committed bytes changed.", retryable: false } }) }, true);
  app.run('capability = "synthetic-download-capability";');
  await app.confirm.fire("click");
  await app.byButton("Download committed redacted report").fire("click");
  const request = app.requests.find(request => request.route.endsWith("/download"));
  assert.equal(request.route, "/exports/" + operationId + "/download");
  assert.equal(request.options.headers.Authorization, "Bearer synthetic-download-capability");
  assert.equal(request.options.credentials, "omit");
  assert.equal(request.options.cache, "no-store");
  assert.equal(app.document.activeElement, app.byId("error"));
  assert.match(app.byId("error").textContent, /committed export is no longer available or its bytes changed/);
  assert.equal(app.byId("status").textContent, "Saved synthetic-output.json.");
  oneExactCommit(app);
});


for (const fails of [false, true]) {
  test("obsolete page " + (fails ? "error" : "success") + " after a dismissed refresh cannot release or overwrite a newer page read", async () => {
    const app = await queueApp({
      "/effects/previews/prev_synthetic000001": () => proposedWrite(),
      "/effects/commits": () => operation("pending", { kind: "commit" }),
      [operationRoute]: () => committedWrite(),
    });
    const parts = pageParts(app); const rows = parts.results.textContent;
    const oldPage = deferred(); app.routes["/review-queue/items"] = () => oldPage.promise;
    const next = app.byButton("Next page"); next.focus();
    const oldAction = next.fire("click"); await settle();
    await app.run('preview(' + JSON.stringify(proposedWrite()) + ')');
    const staged = deferred(); app.routes["/review-queue/items"] = () => staged.promise;
    const commit = app.byButton("Confirm this exact write", app.byId("preview-dialog")).fire("click");
    await settle(); app.byId("preview-dialog").escape(); await settle();
    staged.resolve(queuePage(new URL("http://local/?page_size=50"))); await commit;
    assert.equal(next.getAttribute("aria-disabled"), "false", "Discarded staging must leave visible page controls usable");
    assert.equal(parts.results.textContent, rows);
    const freshPage = deferred(); app.routes["/review-queue/items"] = () => freshPage.promise;
    next.focus(); const freshAction = next.fire("click"); await settle();
    const reads = app.requests.filter(request => request.route === "/review-queue/items").length;
    assert.equal(next.getAttribute("aria-disabled"), "true");
    assert.equal(parts.results.getAttribute("aria-busy"), "true");
    const pendingStatus = parts.status.textContent;
    oldPage.resolve(fails ? { status: 503, body: { code: "internal-error", message: "Obsolete page read.", retryable: true } }
      : queuePage(new URL("http://local/?page_size=50&cursor=synthetic-page-2")));
    await oldAction;
    assert.equal(parts.results.textContent, rows);
    assert.equal(parts.status.textContent, pendingStatus);
    assert.equal(parts.error.hidden, true);
    assert.equal(app.document.activeElement === next, true, "An obsolete response cannot seize newer request focus");
    assert.equal(next.getAttribute("aria-disabled"), "true", "Old finally must not release the newer busy action");
    assert.equal(parts.results.getAttribute("aria-busy"), "true");
    await next.fire("click");
    assert.equal(app.requests.filter(request => request.route === "/review-queue/items").length, reads, "New request keeps its duplicate-activation guard");
    freshPage.resolve(queuePage(new URL("http://local/?page_size=50&cursor=synthetic-page-2"))); await freshAction;
    assert.equal(parts.status.textContent, "53 matching items of 53 total · Page 2.");
    assert.match(parts.results.textContent, /synthetic-51/);
    assert.equal(app.document.activeElement === parts.results, true);
    assert.equal(next.getAttribute("aria-disabled"), "false");
    assert.equal(parts.results.getAttribute("aria-busy"), "false");
    oneExactCommit(app);
  });
}

/** Build intrinsic synthetic metadata using the existing normalized index field order. */
function metadataPreview(count = 1) {
  const index = {schema_version:"forge.workspace/1",label:"Synthetic metadata <tag> π",resources:Array.from({length:count},(_,number)=>({key:`item-${number}`,role:"policy-source",path:`item-${number}.md`}))};
  const bundle = {schema_version:"forge.workspace-index-bundle/1",content_profile:"index-and-hashes",index,
    index_sha256:createHash("sha256").update(JSON.stringify(index,null,2)+"\n").digest("hex"),
    pins:index.resources.map(resource=>({key:resource.key,sha256:"a".repeat(64),size_bytes:1}))};
  return {bundle,snapshot_version:"synthetic-version-1",source_index_present:true,
    included_metadata:["project-label","resource-keys","typed-roles","project-relative-paths","sha256-fingerprints","byte-lengths"],source_content_included:false};
}

/** Build complete synthetic comparison rows without claiming backend execution. */
function metadataComparison(preview, extras = 0, observed = "valid") {
  const count = preview.bundle.pins.length;
  return {scope:"registered-fingerprints-only",snapshot_version:"synthetic-version-1",source_index_present:true,state:"matched",
    current_resources:count+extras,current_only_resources:extras,expected_index_matches_current:extras===0,expected_resources:count,
    matched_resources:count,unregistered_resources:0,mismatched_resources:0,
    items:preview.bundle.pins.map(pin=>({key:pin.key,status:"matched",reason_codes:[],observed_resource_validation_state:observed})),source_content_included:false};
}

/** Install the actual metadata panel through full-asset navigation, with only API responses stubbed. */
async function metadataApp(overrides = {}) {
  const preview = metadataPreview();
  const app = harness({/** Return the default complete synthetic preview for metadata panel probes. */ "/project/bundle-preview":()=>preview,/** Return default complete synthetic comparison rows for metadata panel probes. */ "/project/bundle-verifications":()=>metadataComparison(preview),...overrides});
  await app.run('navigate("Trace & Reports")');
  return app;
}

/** Resolve real rendered metadata controls, statuses and errors by their associations. */
function metadataParts(app) {
  const panel=app.byId("view").querySelector("[data-bundle-panel]");assert(panel);
  return {panel,preview:app.byButton("Preview metadata",panel),download:app.byButton("Download metadata bundle",panel),
    acknowledgment:app.byLabel("I understand that labels, resource keys, paths and hashes can reveal project information."),
    file:app.byLabel("Choose a metadata bundle JSON file"),compare:app.byButton("Compare registered fingerprints",panel),
    previewStatus:panel.querySelector("[data-bundle-preview-status]"),previewError:panel.querySelector("[data-bundle-preview-error]"),
    comparisonStatus:panel.querySelector("[data-bundle-comparison-status]"),comparisonError:panel.querySelector("[data-bundle-comparison-error]"),
    comparison:panel.querySelector("[data-bundle-comparison]")};
}

/** Supply bounded raw file bytes to the native-control double, never to a parsed UI manifest. */
async function chooseMetadataBytes(app, bytes, name = "metadata.json", read) {
  const buffer=Buffer.from(bytes);const file=metadataParts(app).file;
  file.files=[{name,size:buffer.length,
    /** Model File.arrayBuffer without decoding or rewriting its selected byte sequence. */
    async arrayBuffer() { return read ? read() : buffer.buffer.slice(buffer.byteOffset,buffer.byteOffset+buffer.byteLength); }}];
  await file.fire("change");return file;
}

for (const count of [0,1,101,1000]) {
  test(`metadata preview retains all ${count} registrations and gates an exact local download`, async()=>{
    const expected=metadataPreview(count);const app=await metadataApp({/** Supply the full synthetic 0, 1, 101 or 1,000 inventory for disclosure and download checks. */ "/project/bundle-preview":()=>expected});const parts=metadataParts(app);
    app.run("dirty = true");const globalStatus=app.byId("status").textContent;
    assert.equal(app.requests.some(request=>request.route==="/project/bundle-preview"),false,"Panel construction makes no background bundle request");
    parts.download.focus();await parts.download.fire("click");assert.equal(app.document.downloads.length,0);
    parts.preview.focus();await parts.preview.fire("click");assert.equal(app.document.activeElement,parts.preview);
    assert.equal(parts.acknowledgment.checked,false);assert.equal(parts.download.getAttribute("aria-disabled"),"true");
    assert.match(parts.previewStatus.textContent,new RegExp(`Metadata preview: ${count} registered resources`));
    assert.equal(parts.panel.querySelector("[data-bundle-metadata]").querySelectorAll("tbody")[0].children.length,count);
    assert.match(parts.panel.textContent,/Synthetic metadata <tag> π/);
    parts.acknowledgment.checked=true;await parts.acknowledgment.fire("change");parts.download.focus();await parts.download.fire("click");
    assert.equal(app.document.activeElement,parts.download);assert.equal(app.document.downloads.length,1);
    const saved=app.document.downloads[0];assert.equal(saved.filename,"forge-workspace-index-and-hashes.json");
    const body=app.objectURLs.get(saved.url);assert(body);assert.equal(body.type,"application/json");
    assert.equal(await body.text(),JSON.stringify(expected.bundle));
    assert.equal(app.run("dirty"),true);assert.equal(app.byId("status").textContent,globalStatus);
    assert.equal(app.requests.some(request=>/effects\/commits|\/exports/.test(request.route)),false);
    await app.poll();assert.equal(app.objectURLs.has(saved.url),false);assert.deepEqual(app.revokedURLs,[saved.url]);
    await parts.preview.fire("click");assert.equal(parts.acknowledgment.checked,false);assert.equal(parts.download.getAttribute("aria-disabled"),"true");
  });
}

test("raw comparison preserves BOM, invalid UTF8 and duplicate bytes, and explicitly retries a fresh read",async()=>{
  const bodies=[];const app=await metadataApp({/** Capture wrapped raw bytes and deliberately reject them so explicit retries remain observable. */ "/project/bundle-verifications":async(_url,options)=>{
    bodies.push(Buffer.from(await options.body.arrayBuffer()));return {status:400,body:{code:"invalid-request",message:"Synthetic strict parser rejection.",retryable:false}};
  }});const parts=metadataParts(app);
  const raw=Buffer.concat([Buffer.from([0xef,0xbb,0xbf,0xff]),Buffer.from('{"schema_version":1,"schema_version":2,"\\u006b":1,"k":2}')]);
  await chooseMetadataBytes(app,raw);parts.compare.focus();await parts.compare.fire("click");
  const expected=Buffer.concat([Buffer.from('{"bundle":'),raw,Buffer.from('}')]);assert.deepEqual(bodies,[expected]);
  assert.equal(app.document.activeElement,parts.comparisonError);assert.match(parts.comparisonError.textContent,/Synthetic strict parser rejection/);
  parts.compare.focus();await parts.compare.fire("click");assert.deepEqual(bodies,[expected,expected]);
  const requests=app.requests.filter(request=>request.route==="/project/bundle-verifications");
  assert.equal(requests.length,2);assert(requests.every(request=>request.options.headers["Content-Type"]==="application/json"&&!request.options.headers["Idempotency-Key"]));
  assert.equal(app.byId("error").hidden,true);assert.equal(app.document.downloads.length,0);
});

test("raw envelope boundary includes its eleven wrapper bytes and prevents oversized file reads",async()=>{
  let posted;const app=await metadataApp({/** Capture the exact-size envelope and reject synthetic padding without implying an intrinsic-valid maximum bundle. */ "/project/bundle-verifications":async(_url,options)=>{
    posted=Buffer.from(await options.body.arrayBuffer());return {status:400,body:{code:"invalid-request",message:"Synthetic padded input is not an intrinsic bundle.",retryable:false}};
  }});const parts=metadataParts(app);const bytes=Buffer.alloc(1024*1024-11,32);
  await chooseMetadataBytes(app,bytes);await parts.compare.fire("click");assert.equal(posted.length,1024*1024);assert.deepEqual(posted.subarray(10,-1),bytes);
  let reads=0;parts.file.files=[{name:"too-large.json",size:1024*1024-10,
    /** Fail the test if an unsupported size is allocated or read. */
    async arrayBuffer(){reads++;throw new Error("must not read");}}];await parts.file.fire("change");
  await parts.compare.fire("click");assert.equal(reads,0);assert.equal(app.requests.filter(request=>request.route==="/project/bundle-verifications").length,1);
  assert.match(parts.comparisonError.textContent,/11-byte JSON wrapper/);
});

test("changing files fences an old read and its finally while a newer comparison remains busy",async()=>{
  const old=deferred();const fresh=deferred();let posts=0;const app=await metadataApp({/** Hold the newer comparison response and count POSTs while an obsolete file read settles. */ "/project/bundle-verifications":()=>{posts++;return fresh.promise;}});const parts=metadataParts(app);
  const bytes=Buffer.from(JSON.stringify(metadataPreview().bundle));await chooseMetadataBytes(app,bytes,"old.json",()=>old.promise);
  parts.compare.focus();const oldAction=parts.compare.fire("click");await settle();
  await chooseMetadataBytes(app,bytes,"new.json");const newAction=parts.compare.fire("click");await settle();
  assert.equal(posts,1);assert.equal(parts.compare.getAttribute("aria-disabled"),"true");const status=parts.comparisonStatus.textContent;
  old.resolve(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength));await oldAction;
  assert.equal(posts,1);assert.equal(parts.comparisonStatus.textContent,status);assert.equal(parts.compare.getAttribute("aria-disabled"),"true");
  await parts.compare.fire("click");assert.equal(posts,1,"Older finally cannot unlock the newer request");
  fresh.resolve(metadataComparison(metadataPreview()));await newAction;assert.equal(parts.compare.getAttribute("aria-disabled"),"false");assert.match(parts.comparisonStatus.textContent,/1 matched/);
});

for (const transition of ["navigation","stop"]) {
  test(`late metadata preview after ${transition} cannot publish acknowledgment, download or focus`,async()=>{
    const late=deferred();const app=await metadataApp({/** Hold the preview until navigation or shutdown invalidates its ownership. */ "/project/bundle-preview":()=>late.promise,/** Acknowledge synthetic shutdown so the late preview must remain suppressed. */ "/session/shutdown":()=>({})});const parts=metadataParts(app);
    parts.preview.focus();const action=parts.preview.fire("click");await settle();await parts.preview.fire("click");
    assert.equal(app.requests.filter(request=>request.route==="/project/bundle-preview").length,1);
    if(transition==="navigation")await app.run('navigate("Overview")');else await app.byId("confirm-stop").fire("click");
    const focus=app.document.activeElement;const focusCount=app.document.focusHistory.length;const status=app.byId("status").textContent;
    late.resolve(metadataPreview());await action;
    assert.equal(parts.acknowledgment.checked,false);assert.equal(parts.download.getAttribute("aria-disabled"),"true");assert.equal(app.document.downloads.length,0);
    assert.equal(app.document.activeElement,focus);assert.equal(app.document.focusHistory.length,focusCount);assert.equal(app.byId("status").textContent,status);
  });
}

test("dismissed guarded refresh retires a retained panel yet lets its fresh read own the new epoch",async()=>{
  const first=deferred();const app=await metadataApp({/** Hold the original preview across a dismissed refresh to test newer epoch ownership. */ "/project/bundle-preview":()=>first.promise});const parts=metadataParts(app);
  parts.preview.focus();const obsolete=parts.preview.fire("click");await settle();
  const resources=deferred();app.routes["/resources"]=()=>resources.promise;app.run("let keepMetadataRefresh = true");
  const refresh=app.run("renderView(()=>keepMetadataRefresh)");await settle();app.run("keepMetadataRefresh = false");
  resources.resolve({resource_version:"synthetic-version-1",page:{items:[],total_matching:0,next_cursor:null}});assert.equal(await refresh,false);
  assert.equal(parts.panel.isConnected,true);assert.equal(parts.acknowledgment.checked,false);assert.equal(parts.preview.getAttribute("aria-disabled"),"false");
  const newer=deferred();app.routes["/project/bundle-preview"]=()=>newer.promise;parts.preview.focus();const fresh=parts.preview.fire("click");await settle();
  const status=parts.previewStatus.textContent;first.resolve(metadataPreview());await obsolete;
  assert.equal(parts.previewStatus.textContent,status);assert.equal(parts.preview.getAttribute("aria-disabled"),"true");assert.equal(parts.download.getAttribute("aria-disabled"),"true");
  newer.resolve(metadataPreview(101));await fresh;assert.match(parts.previewStatus.textContent,/101 registered resources/);assert.equal(app.document.activeElement,parts.preview);
});

test("independent preview and comparison lanes retain subset extras and stale matched observations",async()=>{
  const late=deferred();const expected=metadataPreview();const app=await metadataApp({/** Hold the preview while the independent comparison lane completes. */ "/project/bundle-preview":()=>late.promise,
    /** Return matched stale fingerprints and one current-only resource without whole-index equality. */ "/project/bundle-verifications":()=>metadataComparison(expected,1,"stale")});const parts=metadataParts(app);
  const preview=parts.preview.fire("click");await settle();await chooseMetadataBytes(app,Buffer.from(JSON.stringify(expected.bundle)));
  parts.compare.focus();await parts.compare.fire("click");assert.match(parts.comparison.textContent,/Whole index matches: no/);assert.match(parts.comparison.textContent,/stale/);
  assert.match(parts.comparisonStatus.textContent,/1 matched.*Current-only registrations: 1/);
  const output=parts.comparison.textContent;late.resolve(expected);await preview;
  assert.equal(parts.comparison.textContent,output);assert.equal(app.document.activeElement,parts.compare);assert.equal(app.byId("error").hidden,true);
});

test("missing current index and explicit empty comparison retain distinct source states",async()=>{
  const empty=metadataPreview(0);const absent={...metadataComparison(empty),source_index_present:false,state:"missing-index",expected_index_matches_current:false};
  const app=await metadataApp({/** Return missing-index for an empty expected inventory while preserving current index absence. */ "/project/bundle-verifications":()=>absent});const parts=metadataParts(app);
  await chooseMetadataBytes(app,Buffer.from(JSON.stringify(empty.bundle)));await parts.compare.fire("click");assert.match(parts.comparison.textContent,/missing-index.*Whole index matches: no/);
  app.routes["/project/bundle-verifications"]=()=>metadataComparison(empty);await parts.compare.fire("click");assert.match(parts.comparison.textContent,/Comparison state: matched.*Whole index matches: yes/);
  assert.match(parts.comparisonStatus.textContent,/0 expected/);
});

test("unsupported metadata and unreconciled comparison cannot become successful downloadable or matched UI",async()=>{
  const hidden={...metadataPreview(),source_content_included:true};const app=await metadataApp({/** Return forbidden source-content metadata so the preview cannot become downloadable. */ "/project/bundle-preview":()=>hidden,
    /** Return an incorrect expected denominator so comparison cannot announce matched results. */ "/project/bundle-verifications":()=>({...metadataComparison(metadataPreview()),expected_resources:2})});const parts=metadataParts(app);
  await parts.preview.fire("click");assert.match(parts.previewError.textContent,/unsupported response/);assert.equal(parts.download.getAttribute("aria-disabled"),"true");
  await chooseMetadataBytes(app,Buffer.from("{}"));await parts.compare.fire("click");assert.match(parts.comparisonError.textContent,/unreconciled/);assert.equal(parts.comparison.children.length,0);
  assert.equal(app.document.downloads.length,0);assert.equal(app.requests.some(request=>request.route==="/effects/commits"),false);
});

test("local metadata failures preserve report edits and do not announce a global saved result",async()=>{
  const app=harness({/** Return a safe missing-index failure while unsaved report edits and global status are retained. */ "/project/bundle-preview":()=>({status:404,body:{code:"not-found",message:"Synthetic absent index.",retryable:false}})});
  app.run("readOnly = false");await app.run('navigate("Trace & Reports")');const parts=metadataParts(app);
  const report=app.byLabel("Report destination within project");report.value="unconfirmed.html";await report.fire("input");
  const status=app.byId("status").textContent;parts.preview.focus();await parts.preview.fire("click");
  assert.equal(app.document.activeElement,parts.previewError);assert.equal(report.value,"unconfirmed.html");assert.equal(app.run("dirty"),true);
  await chooseMetadataBytes(app,Buffer.from("{}"),"unreadable.json",()=>Promise.reject(new Error("PRIVATE FILE CONTENT MUST NOT APPEAR")));
  parts.compare.focus();await parts.compare.fire("click");assert.equal(app.document.activeElement,parts.comparisonError);assert.match(parts.comparisonError.textContent,/chosen file could not be read/);
  assert(!parts.comparisonError.textContent.includes("PRIVATE"));assert.equal(app.requests.some(request=>request.route==="/project/bundle-verifications"),false);
  assert.equal(app.byId("status").textContent,status);assert.equal(app.byId("error").hidden,true);assert.equal(app.document.querySelector("dialog[open]"),null);
});

test("a raw file comparison superseded during request spacing is never sent",async()=>{
  const app=await metadataApp();const parts=metadataParts(app);const bytes=Buffer.from(JSON.stringify(metadataPreview().bundle));
  await chooseMetadataBytes(app,bytes,"paced-old.json");app.run("performance.now = () => 0; nextRequestAt = 1000;");
  parts.compare.focus();const old=parts.compare.fire("click");await settle();assert.equal(app.timers.length,1);
  await chooseMetadataBytes(app,bytes,"replacement.json");const status=parts.comparisonStatus.textContent;
  await app.poll();await old;
  assert.equal(app.requests.some(request=>request.route==="/project/bundle-verifications"),false);
  assert.equal(parts.comparisonStatus.textContent,status);assert.equal(parts.compare.getAttribute("aria-disabled"),"false");assert.equal(parts.comparisonError.hidden,true);
});

test("a failed provenance transition retires disclosure while the retained metadata panel remains reusable",async()=>{
  const app=await metadataApp({/** Fail provenance navigation so disclosure retires while the retained metadata panel remains reusable. */ "/provenance/entries":()=>({status:503,body:{code:"internal-error",message:"Synthetic provenance read failed.",retryable:true}})});const parts=metadataParts(app);
  await parts.preview.fire("click");parts.acknowledgment.checked=true;await parts.acknowledgment.fire("change");
  assert.equal(parts.download.getAttribute("aria-disabled"),"false");
  await assert.rejects(app.run('showProvenance("synthetic-anchor")'),/Synthetic provenance read failed/);
  assert.equal(parts.panel.isConnected,true);assert.equal(parts.acknowledgment.checked,false);assert.equal(parts.download.getAttribute("aria-disabled"),"true");
  parts.preview.focus();await parts.preview.fire("click");assert.equal(app.document.activeElement,parts.preview);assert.match(parts.previewStatus.textContent,/1 registered resources/);
  assert.equal(app.document.downloads.length,0);
});


// API2 source controls retain the original API1 callback bytes above. All new
// replies are synthetic transport observations, not native admission or acceptance.
const lifecycleBootstrap2 = {"forge-api-major":"2", "forge-api-contract-version":"2.0.0"};
const lifecycleOriginalRoles = ["policy-source", "oscal-catalog-artifact", "oscal-component-artifact", "mapping-collection", "applicability-manifest", "applicability-report", "trace-report"];
const lifecycleProfiles = {
  "lifecycle-record":"lifecycle-record-structure",
  "lifecycle-source":"opaque-fingerprint-bytes",
  "oscal-profile-artifact":"native-oscal-schema",
  "oscal-ssp-artifact":"native-oscal-schema",
  "framework-impact-manifest":"framework-impact-manifest",
  "successor-map":"successor-map",
  "framework-impact-report":"framework-impact-prior-admission",
  "framework-impact-dispositions":"framework-impact-dispositions",
};

/** Author the exact current API2 Session envelope without altering historical API1 fixtures. */
function lifecycleSession(readOnly = true, overrides = {}) {
  return {capability:"synthetic-api2-capability-32-characters", session:{
    session_id:"sess_synthetic002", mode:"browser", api_major:2, read_only:readOnly,
    contract_version:"2.0.0", project_label:"Synthetic API2 project", launched_at:"2026-10-02T00:00:00Z",
    ...overrides,
  }};
}

/** Render declared role/profile observations; these fixtures do not validate domain content. */
function lifecycleResources() {
  return [...lifecycleOriginalRoles, ...Object.keys(lifecycleProfiles)].map((role, number) => ({
    resource_id:`res_lifecycle000${number}`, key:`life-${number}`, role, path:`life-${number}.json`,
    sha256:"a".repeat(64), size_bytes:role === "lifecycle-source" ? 0 : 1,
    validation_state:role === "framework-impact-report" ? "stale" : role === "lifecycle-record" ? "invalid" : "valid",
    stale:role === "framework-impact-report", version:"synthetic-version-1",
    ...(Object.hasOwn(lifecycleProfiles, role) ? {validation_profile:lifecycleProfiles[role]} : {}),
  }));
}

/** Build complete paired synthetic metadata, preserving role order and zero-byte fingerprints. */
function lifecycleBundle(indexVersion = "forge.workspace/2", count) {
  assert(["forge.workspace/1", "forge.workspace/2"].includes(indexVersion));
  const roles = indexVersion === "forge.workspace/1" ? lifecycleOriginalRoles : [...lifecycleOriginalRoles, ...Object.keys(lifecycleProfiles)];
  const preview = metadataPreview(0);
  const index = {schema_version:indexVersion, label:"API2 metadata <script> literal label",
    resources:Array.from({length:count ?? roles.length}, (_, number) => ({key:`life-${number}`, role:roles[number % roles.length], path:`life-${number}.json`}))};
  preview.bundle = {schema_version:indexVersion === "forge.workspace/1" ? "forge.workspace-index-bundle/1" : "forge.workspace-index-bundle/2",
    content_profile:"index-and-hashes", index, index_sha256:createHash("sha256").update(JSON.stringify(index, null, 2)+"\n").digest("hex"),
    pins:index.resources.map(resource => ({key:resource.key, sha256:"a".repeat(64), size_bytes:resource.role === "lifecycle-source" ? 0 : 1}))};
  return preview;
}

/** Unlock through real production handlers before issuing API2 project reads in the fake transport. */
async function lifecycleBrowserApp(overrides = {}, readOnly = true) {
  const app = harness({
    /** Return a matching synthetic Session so capability ownership is acquired through the real unlock path. */
    "/session/unlock":() => lifecycleSession(readOnly),
    ...overrides,
  }, lifecycleBootstrap2);
  await beginUnlock(app).action;
  assert.equal(app.run("capability"), "synthetic-api2-capability-32-characters");
  return app;
}

/** Install the actual API2 metadata panel without automatic preview, comparison or disclosure. */
async function lifecycleMetadataApp(preview = lifecycleBundle(), overrides = {}) {
  const app = await lifecycleBrowserApp({
    /** Supply the complete paired metadata fixture only after explicit Preview metadata activation. */
    "/project/bundle-preview":() => preview,
    /** Return complete synthetic fingerprint observations without asserting domain approval. */
    "/project/bundle-verifications":() => metadataComparison(preview),
    ...overrides,
  });
  await app.run('navigate("Trace & Reports")');
  return app;
}

/** Prepare a normal receipt-backed index write; no synthetic response can itself confirm it. */
async function lifecycleWriteApp(major = 2, overrides = {}) {
  const preview = {...proposedWrite(), operation_type:"workspace-index-update",
    target:{status:"replace", path:"forge.workspace.json"}, semantic_summary:"Synthetic index-version preparation."};
  const app = harness({
    /** Use each major's original Session shape; API1's historical version fixture remains unchanged. */
    "/session/unlock":() => major === 2 ? lifecycleSession(false) : unlockedSession(false),
    /** Preparation returns the documented draft envelope, requiring a separate receipt read and confirmation. */
    "/resources/register":() => ({validation:preview.validation, preview}),
    /** Bind the ordinary preview dialog to the same synthetic receipt and proposed target. */
    "/effects/previews/prev_synthetic000001":() => preview,
    /** Acknowledge a synthetic commit; publication is observed separately through its operation ID. */
    "/effects/commits":() => operation("pending", {kind:"commit"}),
    /** Supply the exact target of the synthetic verified write only when confirmation triggers its observation. */
    [operationRoute]:() => committedWrite("forge.workspace.json"),
    ...overrides,
  }, major === 2 ? lifecycleBootstrap2 : {"forge-api-major":"1"});
  await beginUnlock(app).action;
  await app.run('navigate("Policies & Artifacts")');
  return app;
}

/** Set real labelled registration controls and preserve dirty values through preview-only work. */
async function lifecycleRegistrationFields(app, role, indexVersion = "preserve") {
  const resourceRole = app.byLabel("Resource role"); resourceRole.value = role;
  const path = app.byLabel("Project-relative file path"); path.value = "chosen-source.md"; await path.fire("input");
  const key = app.byLabel("Stable resource key"); key.value = "chosen-source"; await key.fire("input");
  if (app.run("apiMajor") === 2) app.byLabel("Index version for this registration").value = indexVersion;
  return {resourceRole, path, key};
}

/** Every observed request must retain its selected namespace; only unlock is sent without a capability. */
function lifecycleRequestOwnership(app, major) {
  assert(app.requests.length > 0);
  for (const request of app.requests) {
    assert.equal(new URL(request.url, "http://127.0.0.1:1").pathname.startsWith(`/api/v${major}/`), true);
    assert.equal(request.options.credentials, "omit");
    assert.equal(request.options.cache, "no-store");
    assert.equal(request.options.redirect, "error");
    assert.equal(request.options.referrerPolicy, "no-referrer");
    assert.equal(request.options.headers.Authorization, request.route === "/session/unlock" ? undefined
      : `Bearer ${major === 2 ? "synthetic-api2-capability-32-characters" : "synthetic-test-capability-32-characters"}`);
  }
}

for (const major of [1, 2]) {
  /** Supported exact bootstrap and matching Session acquire capability before any project request. */
  test(`exact API${major} bootstrap unlocks into its own namespace without changing initial focus`, async () => {
    const app = harness({
      /** Return a matching Session only after observing the unauthenticated unlock request. */
      "/session/unlock":() => major === 2 ? lifecycleSession() : unlockedSession(),
    }, major === 2 ? lifecycleBootstrap2 : {"forge-api-major":"1"});
    assert.equal(app.requests.length, 0);
    assert.equal(app.document.activeElement, app.byId("passphrase"));
    const attempt = beginUnlock(app); await attempt.action;
    assert.equal(attempt.field.value, "");
    assert.equal(app.byId("unlock-panel").hidden, true);
    assert.equal(app.document.activeElement, app.byId("main"));
    assert.equal(app.requests.filter(request => request.route === "/session/unlock").length, 1);
    assert.equal(app.requests.filter(request => request.route === "/project/summary").length, 1);
    lifecycleRequestOwnership(app, major);
  });
}

for (const fixture of [
  {name:"missing major", bootstrap:{}},
  {name:"major meta without content", bootstrap:{"forge-api-major":null}},
  {name:"empty major", bootstrap:{"forge-api-major":""}},
  {name:"unsupported major", bootstrap:{"forge-api-major":"3"}},
  {name:"decimal-coerced major", bootstrap:{...lifecycleBootstrap2, "forge-api-major":"2.0"}},
  {name:"exponent-coerced major", bootstrap:{...lifecycleBootstrap2, "forge-api-major":"2e0"}},
  {name:"whitespace-coerced major", bootstrap:{...lifecycleBootstrap2, "forge-api-major":" 2 "}},
  {name:"missing API2 version", bootstrap:{"forge-api-major":"2"}},
  {name:"unsupported current API2 version", bootstrap:{...lifecycleBootstrap2, "forge-api-contract-version":"2.9.9"}},
  {name:"whitespace API2 version", bootstrap:{...lifecycleBootstrap2, "forge-api-contract-version":"2.0.0 "}},
]) {
  /** Reject malformed shell selection before sending a passphrase or granting capability ownership. */
  test(`bootstrap rejects ${fixture.name} before every transport request`, async () => {
    const app = harness({}, fixture.bootstrap);
    const initialWorkspaceHidden = app.byId("workspace").hidden;
    const attempt = beginUnlock(app); await attempt.action;
    assert.equal(app.requests.length, 0);
    assert.equal(app.run("capability"), "");
    assert.equal(app.byId("unlock-panel").hidden, false);
    assert.equal(app.byId("workspace").hidden, initialWorkspaceHidden);
    assert.equal(attempt.field.value, "");
    assert.equal(app.run("unlockPending"), false);
    assert.equal(attempt.form.getAttribute("aria-busy"), "false");
    assert.equal(app.document.activeElement, app.byId("error"));
    assert.match(app.byId("error").textContent, /matching supported workspace API assets/);
  });
}

for (const fixture of [
  {name:"foreign major", session:{api_major:1, contract_version:"1.2.0"}},
  {name:"string major", session:{api_major:"2", contract_version:"2.0.0"}},
  {name:"unsupported Session version", session:{api_major:2, contract_version:"2.0.1"}},
  {name:"missing Session version", session:{api_major:2, contract_version:undefined}},
]) {
  /** A mismatched Session response cannot authorize a project read or become a successful unlock. */
  test(`API2 rejects ${fixture.name} before retaining the returned capability`, async () => {
    const app = harness({
      /** Return the selected mismatch as an unlock observation without granting project authority. */
      "/session/unlock":() => lifecycleSession(true, fixture.session),
    }, lifecycleBootstrap2);
    const hidden = app.byId("workspace").hidden;
    const attempt = beginUnlock(app); await attempt.action;
    assert.equal(app.requests.length, 1);
    assert.equal(app.requests[0].options.headers.Authorization, undefined);
    assert.equal(app.run("capability"), "");
    assert.equal(app.byId("workspace").hidden, hidden);
    assert.equal(app.byId("unlock-panel").hidden, false);
    assert.equal(app.document.activeElement, app.byId("error"));
    assert.match(app.byId("error").textContent, /requires the selected API version/);
    assert.equal(attempt.field.value, "");
    assert.equal(attempt.submit.getAttribute("aria-disabled"), "false");
  });
}

/** Original-major pages also reject a foreign Session before acquiring its capability. */
test("API1 refuses an API2 Session while preserving the original locked state", async () => {
  const app = harness({/** Return a foreign Session through the original-major unlock route. */ "/session/unlock":() => lifecycleSession()});
  await beginUnlock(app).action;
  assert.equal(app.requests.length, 1);
  assert.equal(app.requests[0].url, "/api/v1/session/unlock");
  assert.equal(app.run("capability"), "");
  assert.equal(app.byId("unlock-panel").hidden, false);
  assert.equal(app.document.activeElement, app.byId("error"));
});

/** API2 displays all declared admission profiles without converting invalid or stale observations into approval. */
test("API2 read-only resources display fifteen roles and paired profiles without write controls", async () => {
  const rows = lifecycleResources();
  const app = await lifecycleBrowserApp({
    /** Return exact synthetic Resource observations; content is never parsed by this source fixture. */
    "/resources":() => ({resource_version:"synthetic-version-1", page:{items:rows, total_matching:rows.length, next_cursor:null}}),
  });
  await app.run('navigate("Policies & Artifacts")');
  const table = app.byId("view").querySelector('[aria-label="Registered project files"]').querySelector("table");
  assert.deepEqual(table.querySelectorAll("th").map(cell => cell.textContent), ["Key", "Role", "Path", "Validation", "Admission profile"]);
  const displayed = table.querySelector("tbody").children;
  assert.equal(displayed.length, 15);
  for (let number = 0; number < rows.length; number++) {
    const row = rows[number];
    assert.deepEqual(displayed[number].children.map(cell => cell.textContent), [row.key, row.role, row.path, row.validation_state, row.validation_profile ?? "—"]);
  }
  assert.equal(app.byId("view").querySelectorAll("form").length, 0);
  assert.match(app.byId("view").textContent, /This session is read-only/);
  assert.equal(app.requests.some(request => request.route === "/resources/register" || request.route === "/effects/commits"), false);
  lifecycleRequestOwnership(app, 2);
});

/** Writable API2 selectors include eight explicit new roles while original-major selectors stay seven-role only. */
test("API2 registration and upload selectors expose fifteen roles without altering API1 choices", async () => {
  for (const major of [1, 2]) {
    const app = await lifecycleWriteApp(major);
    const expected = major === 2 ? [...lifecycleOriginalRoles, ...Object.keys(lifecycleProfiles)] : lifecycleOriginalRoles;
    for (const label of ["Resource role", "Uploaded resource role"]) {
      assert.deepEqual(app.byLabel(label).querySelectorAll("option").map(option => option.value), expected);
    }
    const hasMigration = app.byId("view").querySelectorAll("button").some(button => button.textContent === "Preview migration to workspace index /2");
    assert.equal(hasMigration, major === 2);
    assert.equal(app.byId("view").querySelectorAll("label").some(label => label.textContent === "Index version for this registration"), major === 2);
    if (major === 2) assert.match(app.byId("view").textContent, /do not establish current freshness, dependency closure or reviewer authority/);
    assert.equal(app.requests.some(request => request.route === "/resources/register"), false);
  }
});

for (const fixture of [
  {name:"original API1 body", major:1, role:"policy-source", index:"preserve"},
  {name:"API2 preserves current index", major:2, role:"policy-source", index:"preserve"},
  {name:"API2 explicitly selects index2", major:2, role:"lifecycle-source", index:"forge.workspace/2"},
]) {
  /** User selection changes only the explicit request variant; preparation itself cannot confirm a write. */
  test(`registration ${fixture.name} uses the normal receipt preview and leaves edits unsaved`, async () => {
    const app = await lifecycleWriteApp(fixture.major);
    const fields = await lifecycleRegistrationFields(app, fixture.role, fixture.index);
    const invoker = app.byButton("Preview registration"); invoker.focus(); await invoker.fire("click");
    const writes = app.requests.filter(request => request.route === "/resources/register");
    assert.equal(writes.length, 1); assert.equal(writes[0].options.method, "POST");
    const expected = {role:fixture.role, path:"chosen-source.md", key:"chosen-source"};
    if (fixture.index === "forge.workspace/2") expected.index_schema_version = "forge.workspace/2";
    assert.deepEqual(JSON.parse(writes[0].options.body), expected);
    assert.match(writes[0].options.headers["Idempotency-Key"], /^[a-f0-9-]{36}$/);
    const dialog = app.byId("preview-dialog"); assert.equal(dialog.open, true);
    assert.equal(app.document.activeElement, dialog.querySelector("h2"));
    assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
    assert.equal(app.run("dirty"), true); assert.doesNotMatch(app.byId("status").textContent, /^Saved /);
    await app.byButton("Keep editing", dialog).fire("click");
    assert.equal(fields.path.isConnected, true); assert.equal(fields.path.value, "chosen-source.md");
    assert.equal(fields.key.value, "chosen-source"); assert.equal(app.run("dirty"), true);
    assert.equal(app.document.activeElement === invoker, true,
      "Keep editing must restore the connected initiating registration control.");
    lifecycleRequestOwnership(app, fixture.major);
  });
}

/** Migration sends only its closed variant, then waits for an exact receipt confirmation and verified result. */
test("API2 migration ignores registration fields and cannot publish Saved before exact confirmation", async () => {
  const app = await lifecycleWriteApp();
  await lifecycleRegistrationFields(app, "lifecycle-source", "forge.workspace/2");
  const migration = app.byButton("Preview migration to workspace index /2"); migration.focus(); await migration.fire("click");
  const request = app.requests.find(request => request.route === "/resources/register");
  assert.deepEqual(JSON.parse(request.options.body), {migration:{from:"forge.workspace/1", to:"forge.workspace/2"}});
  assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
  assert.equal(app.run("dirty"), true);
  app.document.holdCloseEvents = true;
  const confirm = app.byButton("Confirm this exact write", app.byId("preview-dialog")); confirm.focus();
  const write = confirm.fire("click"); await settle();
  assert.equal(app.document.closeEvents.length, 1);
  assert.doesNotMatch(app.byId("status").textContent, /^Saved /);
  app.document.closeEvents.shift()(); await write;
  oneExactCommit(app);
  assert.equal(app.run("dirty"), false);
  assert.equal(app.byId("status").textContent, "Saved forge.workspace.json.");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  lifecycleRequestOwnership(app, 2);
});

for (const fixture of [
  {version:"forge.workspace/1", count:7},
  {version:"forge.workspace/2", count:15},
  {version:"forge.workspace/2", count:1000},
  {version:"forge.workspace/2", count:0},
]) {
  /** API2 preserves complete paired versions, including an explicit empty index, through local acknowledgment and download. */
  test(`API2 metadata ${fixture.version} with ${fixture.count} resources requires fresh acknowledgment`, async () => {
    const expected = lifecycleBundle(fixture.version, fixture.count);
    const app = await lifecycleMetadataApp(expected); const parts = metadataParts(app);
    app.run("dirty = true"); const status = app.byId("status").textContent;
    parts.preview.focus(); await parts.preview.fire("click");
    assert.equal(app.document.activeElement, parts.preview);
    assert.match(parts.previewStatus.textContent, new RegExp(`${fixture.count} registered resources`));
    assert.equal(parts.acknowledgment.checked, false);
    await parts.download.fire("click"); assert.equal(app.document.downloads.length, 0);
    assert.equal(parts.panel.querySelector('[data-bundle-metadata]').querySelector("tbody").children.length, fixture.count);
    assert.match(parts.panel.textContent, /API2 metadata <script> literal label/);
    assert.equal(parts.panel.querySelectorAll("script").length, 0);
    parts.acknowledgment.checked = true; await parts.acknowledgment.fire("change");
    parts.download.focus(); await parts.download.fire("click");
    assert.equal(app.document.activeElement, parts.download);
    assert.equal(app.document.downloads.length, 1);
    const download = app.document.downloads[0]; assert.equal(download.filename, "forge-workspace-index-and-hashes.json");
    assert.equal(await app.objectURLs.get(download.url).text(), JSON.stringify(expected.bundle));
    assert.equal(app.run("dirty"), true); assert.equal(app.byId("status").textContent, status);
    assert.equal(app.requests.some(request => /effects\/commits|\/exports|\/resources\/register/.test(request.route)), false);
    await parts.preview.fire("click");
    assert.equal(parts.acknowledgment.checked, false); assert.equal(parts.download.getAttribute("aria-disabled"), "true");
    assert.equal(app.objectURLs.has(download.url), false);
    assert.deepEqual(app.revokedURLs, [download.url]);
    lifecycleRequestOwnership(app, 2);
  });
}

for (const defect of ["default-major-bundle2", "bundle1-index2", "bundle2-index1", "index1-new-role", "unknown-role", "unknown-field", "wrong-profile", "pin-order", "index3", "1001-registrations"]) {
  /** Invalid version pairing or closed metadata cannot become local disclosure authority or a partial download. */
  test(`metadata rejects ${defect} without retaining acknowledgment or downloadable bytes`, async () => {
    const expected = lifecycleBundle(defect === "index1-new-role" ? "forge.workspace/1" : "forge.workspace/2", defect === "1001-registrations" ? 1001 : undefined);
    if (defect === "bundle1-index2") expected.bundle.schema_version = "forge.workspace-index-bundle/1";
    if (defect === "bundle2-index1") expected.bundle.index.schema_version = "forge.workspace/1";
    if (defect === "index1-new-role") expected.bundle.index.resources[0].role = "lifecycle-record";
    if (defect === "unknown-role") expected.bundle.index.resources[0].role = "unapproved-role";
    if (defect === "unknown-field") expected.bundle.index.resources[0].validation_profile = "opaque-fingerprint-bytes";
    if (defect === "wrong-profile") expected.bundle.content_profile = "source-content";
    if (defect === "pin-order") expected.bundle.pins.reverse();
    if (defect === "index3") expected.bundle.index.schema_version = "forge.workspace/3";
    let app;
    if (defect === "default-major-bundle2") {
      app = harness({
        /** Acquire original-major capability through the unchanged unlock handler before the version rejection probe. */
        "/session/unlock":() => unlockedSession(),
        /** Supply bundle2 to an original-major panel so the closed version guard must reject it. */
        "/project/bundle-preview":() => expected,
      });
      await beginUnlock(app).action;
      await app.run('navigate("Trace & Reports")');
    } else app = await lifecycleMetadataApp(expected);
    const parts = metadataParts(app); parts.preview.focus(); await parts.preview.fire("click");
    assert.equal(app.document.activeElement, parts.previewError);
    assert.match(parts.previewError.textContent, /unsupported response/);
    assert.equal(parts.acknowledgment.checked, false);
    assert.equal(parts.download.getAttribute("aria-disabled"), "true");
    assert.equal(parts.panel.querySelector('[data-bundle-metadata]').children.length, 0);
    parts.acknowledgment.checked = true; await parts.acknowledgment.fire("change"); await parts.download.fire("click");
    assert.equal(parts.acknowledgment.checked, false);
    assert.equal(app.document.downloads.length, 0); assert.equal(app.objectURLs.size, 0);
    assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
  });
}

/** API2 raw comparisons keep decoder-invalid bytes intact and retry a fresh read without an idempotency key. */
test("API2 comparison wraps the exact chosen bytes and keeps raw-parser failures local", async () => {
  const bodies = [];
  const app = await lifecycleMetadataApp(lifecycleBundle(), {
    /** Observe exact transport bytes and return a strict-parser failure without reflecting supplied content. */
    "/project/bundle-verifications":async (_url, options) => {
      bodies.push(Buffer.from(await options.body.arrayBuffer()));
      return {status:400, body:{code:"invalid-request", message:"Synthetic API2 raw parser rejection.", retryable:false}};
    },
  });
  const parts = metadataParts(app);
  const bytes = Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf, 0xff]), Buffer.from('{"schema_version":"forge.workspace-index-bundle/2","schema_version":2,"\\u006b":1,"k":2}')]);
  await chooseMetadataBytes(app, bytes, "selected-api2.json");
  parts.compare.focus(); await parts.compare.fire("click");
  const expected = Buffer.concat([Buffer.from('{"bundle":'), bytes, Buffer.from('}')]);
  assert.deepEqual(bodies, [expected]);
  assert.equal(app.document.activeElement, parts.comparisonError);
  assert.match(parts.comparisonError.textContent, /Synthetic API2 raw parser rejection/);
  parts.compare.focus(); await parts.compare.fire("click"); assert.deepEqual(bodies, [expected, expected]);
  const requests = app.requests.filter(request => request.route === "/project/bundle-verifications");
  assert.equal(requests.length, 2);
  assert(requests.every(request => request.options.method === "POST" && request.options.headers["Content-Type"] === "application/json" && !request.options.headers["Idempotency-Key"]));
  assert.equal(app.document.downloads.length, 0); assert.equal(app.byId("error").hidden, true);
  lifecycleRequestOwnership(app, 2);
});

/** Supplied index1 matches only expected fingerprints while extra current index2 registrations remain distinct from approval. */
test("API2 comparison separates old-bundle subset matches, new-role extras and invalid content observations", async () => {
  const expected = lifecycleBundle("forge.workspace/1");
  const app = await lifecycleMetadataApp(lifecycleBundle(), {
    /** Observe seven expected matches and eight current-only registrations without asserting whole-index equality. */
    "/project/bundle-verifications":() => metadataComparison(expected, 8, "invalid"),
  });
  const parts = metadataParts(app); const status = app.byId("status").textContent;
  await chooseMetadataBytes(app, Buffer.from(JSON.stringify(expected.bundle)), "old-version.json");
  parts.compare.focus(); await parts.compare.fire("click");
  assert.equal(app.document.activeElement, parts.compare);
  assert.match(parts.comparison.textContent, /Comparison state: matched\. Whole index matches: no/);
  assert.match(parts.comparison.textContent, /invalid/);
  assert.match(parts.comparisonStatus.textContent, /7 matched.*of 7 expected.*Current-only registrations: 8/);
  assert.equal(parts.comparison.querySelector("tbody").children.length, 7);
  assert.equal(app.byId("status").textContent, status); assert.equal(parts.acknowledgment.checked, false);
  assert.equal(app.document.downloads.length, 0);
  lifecycleRequestOwnership(app, 2);
});

/** An obsolete API2 file read and finally cannot send bytes or unlock a newer comparison lane. */
test("API2 changing files fences old reads while newer comparison owns busy state", async () => {
  const obsolete = deferred(); const fresh = deferred(); let posts = 0;
  const expected = lifecycleBundle();
  const app = await lifecycleMetadataApp(expected, {
    /** Hold the current comparison response while the prior selected file's read remains unresolved. */
    "/project/bundle-verifications":() => { posts++; return fresh.promise; },
  });
  const parts = metadataParts(app); const bytes = Buffer.from(JSON.stringify(expected.bundle));
  await chooseMetadataBytes(app, bytes, "obsolete-v2.json", () => obsolete.promise);
  parts.compare.focus(); const oldRead = parts.compare.fire("click"); await settle();
  await chooseMetadataBytes(app, bytes, "current-v2.json"); const currentRead = parts.compare.fire("click"); await settle();
  assert.equal(posts, 1); const status = parts.comparisonStatus.textContent;
  obsolete.resolve(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset+bytes.byteLength)); await oldRead;
  assert.equal(posts, 1); assert.equal(parts.comparisonStatus.textContent, status);
  assert.equal(parts.compare.getAttribute("aria-disabled"), "true");
  await parts.compare.fire("click"); assert.equal(posts, 1);
  fresh.resolve(metadataComparison(expected)); await currentRead;
  assert.equal(parts.compare.getAttribute("aria-disabled"), "false");
  assert.match(parts.comparisonStatus.textContent, /15 matched/);
  lifecycleRequestOwnership(app, 2);
});

for (const transition of ["navigation", "stop"]) {
  /** Current-major selection cannot revive metadata, acknowledgment or focus after its installed view is retired. */
  test(`API2 late preview and comparison after ${transition} cannot publish or steal focus`, async () => {
    const previewReply = deferred(); const comparisonReply = deferred(); const expected = lifecycleBundle();
    const app = await lifecycleMetadataApp(expected, {
      /** Delay the preview across the chosen session or view transition. */
      "/project/bundle-preview":() => previewReply.promise,
      /** Delay a complete registered comparison across the same transition. */
      "/project/bundle-verifications":() => comparisonReply.promise,
      /** Acknowledge synthetic shutdown through API2 without adding a project effect. */
      "/session/shutdown":() => ({}),
    });
    const parts = metadataParts(app);
    parts.preview.focus(); const previewRead = parts.preview.fire("click"); await settle();
    await chooseMetadataBytes(app, Buffer.from(JSON.stringify(expected.bundle)), "late-api2.json");
    parts.compare.focus(); const comparisonRead = parts.compare.fire("click"); await settle();
    if (transition === "navigation") await app.run('navigate("Overview")');
    else { await app.byId("stop").fire("click"); await app.byId("confirm-stop").fire("click"); }
    const focus = app.document.activeElement; const focusCount = app.document.focusHistory.length;
    const status = app.byId("status").textContent; const requests = app.requests.length;
    previewReply.resolve(expected); comparisonReply.resolve(metadataComparison(expected));
    await previewRead; await comparisonRead;
    assert.equal(app.document.activeElement, focus); assert.equal(app.document.focusHistory.length, focusCount);
    assert.equal(app.byId("status").textContent, status); assert.equal(app.requests.length, requests);
    assert.equal(parts.acknowledgment.checked, false); assert.equal(parts.download.getAttribute("aria-disabled"), "true");
    assert.equal(parts.comparison.children.length, 0); assert.equal(app.document.downloads.length, 0);
    if (transition === "stop") assert.equal(app.run("capability"), "");
    lifecycleRequestOwnership(app, 2);
  });
}

/** Cancelling an unsaved transition preserves API2 disclosure ownership; explicit discard retires it and returns destination focus. */
test("API2 dirty navigation cancellation retains the installed acknowledged metadata preview", async () => {
  const app = await lifecycleMetadataApp(); const parts = metadataParts(app);
  await parts.preview.fire("click"); parts.acknowledgment.checked = true; await parts.acknowledgment.fire("change");
  app.run("dirty = true"); const invoker = app.byButton("Overview", app.byId("navigation")); invoker.focus();
  const cancelled = invoker.fire("click"); await settle();
  await app.byButton("Keep editing", app.document.querySelector("dialog[open]")).fire("click"); await cancelled;
  assert.equal(parts.panel.isConnected, true); assert.equal(parts.acknowledgment.checked, true);
  assert.equal(parts.download.getAttribute("aria-disabled"), "false"); assert.equal(app.run("dirty"), true);
  assert.equal(app.document.activeElement, invoker);
  const discarded = invoker.fire("click"); await settle();
  await app.byButton("Discard edits", app.document.querySelector("dialog[open]")).fire("click"); await discarded;
  assert.equal(parts.panel.isConnected, false); assert.equal(parts.acknowledgment.checked, false);
  assert.equal(app.run("dirty"), false); assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.equal(app.byId("view-title").textContent, "Overview"); assert.equal(app.document.downloads.length, 0);
  lifecycleRequestOwnership(app, 2);
});

/** Confirmed report downloads must keep the selected API2 namespace and the existing authenticated binary transport options. */
test("API2 confirmed redacted export download never falls back to the foreign API1 namespace", async () => {
  const report = new Blob(["synthetic redacted report"], {type:"text/html"});
  const exportPreview = {...proposedWrite(), operation_type:"report-export", target:{status:"create", path:"synthetic-output.html"}};
  const app = await lifecycleWriteApp(2, {
    /** Supply a normal succeeded export preparation containing its receipt-bound preview, without confirming bytes. */
    "/exports":() => operation("succeeded", {kind:"export", result:{operation_id:operationId, preview:exportPreview,
      redaction_summary:{removed_categories:["reviewer-names", "absolute-paths", "source-excerpts", "secrets"]}}}),
    /** Return the same prepared export when the actual operation row opens its receipt dialog. */
    "/effects/previews/prev_synthetic000001":() => exportPreview,
    /** Observe the confirmed synthetic HTML target only after the actual receipt confirmation request. */
    [operationRoute]:() => committedWrite("synthetic-output.html"),
    /** Return only an explicit bounded binary fixture at the selected major's confirmed export download route. */
    [`/exports/${operationId}/download`]:() => ({download_blob:report}),
  });
  await app.run('navigate("Trace & Reports")');
  app.byLabel("Report").value = "trace";
  const target = app.byLabel("Report destination within project"); target.value = "synthetic-output.html"; await target.fire("input");
  const invoker = app.byButton("Prepare export"); invoker.focus(); await invoker.fire("click");
  assert.deepEqual(JSON.parse(app.requests.find(request => request.route === "/exports").options.body),
    {report_kind:"trace", target_path:"synthetic-output.html", format:"static-html", redaction_profile:"strict-default"});
  assert.equal(app.byId("preview-dialog").open, true);
  assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
  await app.byButton("Confirm this exact write", app.byId("preview-dialog")).fire("click");
  oneExactCommit(app);
  assert.equal(app.requests.some(request => request.route.endsWith("/download")), false);
  const download = app.byButton("Download committed redacted report"); download.focus(); await download.fire("click");
  const requests = app.requests.filter(request => request.route.endsWith("/download"));
  assert.equal(requests.length, 1); assert.equal(requests[0].url, `/api/v2/exports/${operationId}/download`);
  assert.equal(requests[0].options.headers.Authorization, "Bearer synthetic-api2-capability-32-characters");
  assert.equal(app.document.activeElement === download, true,
    "Authenticated download must retain its connected initiating control focus.");
  assert.equal(app.document.downloads.length, 1);
  const retained = app.document.downloads[0]; assert.equal(retained.filename, "forge-redacted-report.html");
  assert.equal(app.objectURLs.get(retained.url), report);
  lifecycleRequestOwnership(app, 2);
});

/** A pending registration keeps its native trigger focused and rejects repeated activation before any second effect request. */
test("busy registration preserves trigger focus and blocks duplicate preparation until its exact preview arrives", async () => {
  const pending = deferred();
  const app = await lifecycleWriteApp(2, {
    /** Hold the actual registration reply while repeat activation exercises the production busy guard. */
    "/resources/register":() => pending.promise,
  });
  await lifecycleRegistrationFields(app, "lifecycle-source", "forge.workspace/2");
  const trigger = app.byButton("Preview registration"); trigger.focus();
  const first = trigger.fire("click"); await settle();
  assert.equal(trigger.disabled, false);
  assert.equal(trigger.getAttribute("aria-disabled"), "true");
  assert.equal(trigger.getAttribute("aria-busy"), "true");
  assert.equal(app.document.activeElement === trigger, true, "pending trigger retains focus");
  await trigger.fire("click");
  assert.equal(app.requests.filter(request => request.route === "/resources/register").length, 1);
  const preview = app.routes["/effects/previews/prev_synthetic000001"]();
  pending.resolve({validation:preview.validation, preview}); await first;
  assert.equal(trigger.getAttribute("aria-disabled"), "false");
  assert.equal(trigger.getAttribute("aria-busy"), "false");
  const dialog = app.byId("preview-dialog"); assert.equal(dialog.open, true);
  await app.byButton("Keep editing", dialog).fire("click");
  assert.equal(dialog.open, false);
  assert.equal(app.document.activeElement === trigger, true, "dismissal restores the connected trigger");
  assert.equal(app.run("dirty"), true);
  assert.equal(app.requests.some(request => request.route === "/effects/commits"), false);
});

// S3 TEMP proposal controls execute the whole selected production asset through the
// existing fake-DOM transport. They do not validate domain fixtures or prove native
// keyboard, layout, offline, AT, privacy or human acceptance.
const s3RecordId = "res_record000001";
const s3ComparisonId = "res_compare000001";
const s3Snapshot = "e".repeat(64);
const s3ComparisonVersion = "f".repeat(64);
const s3FindingId = "11111111-1111-5111-8111-111111111111";
const s3OtherFindingId = "22222222-2222-5222-8222-222222222222";
const s3Bootstrap = {"forge-api-major":"2", "forge-api-contract-version":"2.1.0"};

/** Supply every complete ChangeSummary field, independently from emitted filter matches. */
function s3Summary(overrides = {}) {
  return {old_controls:1, new_controls:1, added:0, removed:0, content_changed:1, identity_migrated:0, unchanged:0,
    findings:2, blocking:1, review_required:1, informational:0, dispositioned_resolved:1,
    dispositioned_accepted_risk:0, dispositioned_still_open:0, undispositioned:1, ...overrides};
}

/** Build declared old/new fingerprint metadata, with poisoned prose intentionally outside the allowlist. */
function s3Fingerprint(id, hash = "a") {
  return {resource_id:id, resource_type:"catalog", raw_sha256:hash.repeat(64), root_uuid:"11111111-1111-5111-8111-111111111111",
    document_version:"1", oscal_version:"1.2.3", resolved_catalog_sha256:null, resolved_catalog_resource_id:null,
    title:"MUST_NOT_RENDER_TITLE", href:"MUST_NOT_RENDER_HREF"};
}

/** Construct bounded page envelopes with distinct entity counts and explicit capture context. */
function s3Page(kind, items, extras = {}) {
  const counts = {records:{registered_records:items.length, matching_records:items.length, unavailable_records:0},
    history:{total_events:items.length}, queue:{distinct_records:1, total_owner_placements:items.length, matching_owner_placements:items.length, total_groups:1, matching_groups:items.length?1:0},
    comparisons:{registered_comparisons:items.length, unavailable_comparisons:0}, changes:{total_changes:items.length, matching_changes:items.length},
    findings:{total_findings:2, matching_findings:items.length}, prior:{total_prior_dispositions:items.length}}[kind];
  const page = {resource_version:createHash("sha256").update("s3-page-"+kind).digest("hex"), snapshot_version:s3Snapshot, availability:"available", as_of:null,
    page:{items, total_matching:items.length, next_cursor:null}, counts,
    ...(["changes", "findings", "prior"].includes(kind) ? {comparison_id:s3ComparisonId, comparison_version:s3ComparisonVersion} : {}),
    ...(kind === "findings" ? {full_summary:s3Summary(), emitted_dispositions:{resolved:items.some(item=>item.disposition)?1:0, accepted_risk:0, still_open:0, undispositioned:items.filter(item=>!item.disposition).length}} : {}),
    ...extras};
  return page;
}

/** Supply one stored record; derived state remains absent until the authored response receives an explicit date. */
function s3Record(id = s3RecordId, asOf = null) {
  return {record_id:id, record_version:"a".repeat(64), resource_id:id, policy_key:"policy-key", version_key:"v1", state:"approved",
    derived_status:asOf ? "approved" : null, owner_keys:["owner-A", "owner-B"], next_review_date:"2026-10-03",
    availability:"valid", diagnostic_code:null, validation_state:"valid", title:"MUST_NOT_RENDER_TITLE", rationale:"MUST_NOT_RENDER_RATIONALE"};
}

/** Provide metadata-only lifecycle detail at the exact date requested by the source consumer. */
function s3RecordDetail(url, extras = {}) {
  return {resource_version:"b".repeat(64), snapshot_version:s3Snapshot, record_id:s3RecordId, resource_id:s3RecordId,
    policy_key:"policy-key", version_key:"v1", as_of:url.searchParams.get("as_of"), state:"approved", derived_status:"approved",
    owner_keys:["owner-A", "owner-B"], next_review_date:"2026-10-03", blockers:[],
    current_fingerprints:{source_sha256:"a".repeat(64), generated_artifacts:[{path:"policy.json", sha256:"b".repeat(64)}]},
    approved_fingerprints:null, artifact_identity_changes:[], replaced_by:null, replacement_record_id:null,
    impact_references:[{finding_id:s3FindingId, binding:"unresolved"}], provenance:[],
    trust_boundary:"actor identities and authority are declared locally and are not authenticated by FORGE",
    title:"MUST_NOT_RENDER_TITLE", rationale:"MUST_NOT_RENDER_RATIONALE", parties:[{name:"MUST_NOT_RENDER_PARTY"}], ...extras};
}

/** Provide fixed comparison context and complete unfiltered summary for all child pages. */
function s3ComparisonDetail(extras = {}) {
  return {resource_version:s3ComparisonVersion, snapshot_version:s3Snapshot, comparison_id:s3ComparisonId, resource_id:s3ComparisonId,
    freshness:"captured-current", old:s3Fingerprint("res_oldcatalog001"), new:s3Fingerprint("res_newcatalog001", "b"), summary:s3Summary(),
    provenance:[], prior_report_sha256:"c".repeat(64), prior_report_admission:"limited-structural",
    trust_boundary:"review dispositions and migration assertions are declared locally; FORGE does not authenticate reviewers or approve risk",
    rationale:"MUST_NOT_RENDER_RATIONALE", ...extras};
}

/** Provide pair-scoped findings with different owners/groups and one declared disposition. */
function s3Findings() {
  return ["A", "B"].map((suffix, number) => ({finding_id:number ? s3OtherFindingId : s3FindingId, comparison_id:s3ComparisonId,
    comparison_version:s3ComparisonVersion, priority:number ? "review-required" : "blocking", reason_code:"control_content_changed",
    required_action:"review-control-change", subject_id:"control-1", change_class:"content-changed", old_sha256:"a".repeat(64), new_sha256:"b".repeat(64),
    old_subjects:[{id:"control-1", sha256:"a".repeat(64)}], new_subjects:[{id:"control-1", sha256:"b".repeat(64)}], migration:null,
    framework_groups:[number ? "group-B" : "group-A"], affected_artifact_id:null, dependency_id:null, policy_resource_identity:"policy-key",
    prior_gap_classification:"applicable-mapped", prior_decision_state:"applicable", owner:"owner-"+suffix, policy_sources:["source-"+suffix],
    disposition:number ? null : {finding_id:s3FindingId, status:"resolved", decided_by:"declared-reviewer", decided_at:"2026-10-01T00:00:00Z", rationale:"MUST_NOT_RENDER_RATIONALE"},
    source_excerpt:"MUST_NOT_RENDER_SOURCE"}));
}

/** Author all nine synthetic query routes; these are renderer controls, not valid domain registrations. */
function s3Routes() {
  return {
    /** Match the new shell contract through the real unlock handler before any authenticated query. */
    "/session/unlock":()=>lifecycleSession(true, {contract_version:"2.1.0"}),
    /** Inventory computes derived metadata only when its explicit query date is supplied. */
    "/lifecycle/records":url=>s3Page("records", [s3Record(s3RecordId, url.searchParams.get("as_of"))], {as_of:url.searchParams.get("as_of")}),
    /** Echo the explicit date for a selected registered lifecycle record. */
    ["/lifecycle/records/"+s3RecordId]:url=>s3RecordDetail(url),
    /** History keeps declared assertion/finding metadata without computed date or prose. */
    ["/lifecycle/records/"+s3RecordId+"/history"]:()=>s3Page("history", [{event_id:"44444444-4444-5444-8444-444444444444", sequence:1, timestamp:"2026-10-01T00:00:00Z", previous_state:"in-review", next_state:"approved", actor_key:"actor-A", declared_role:"approver", assertions:[{actor_key:"actor-A", declared_role:"approver"}], impact_references:[{finding_id:s3FindingId, binding:"unresolved"}], replacement:null, rationale:"MUST_NOT_RENDER_RATIONALE"}], {record_id:s3RecordId}),
    /** Owner placements deliberately outnumber distinct records and groups. */
    "/lifecycle/queue":url=>s3Page("queue", ["owner-A", "owner-B"].map(owner_key=>({record_id:s3RecordId, resource_id:s3RecordId, owner_key, next_review_date:"2026-10-03", policy_key:"policy-key", version_key:"v1", state:"approved", derived_status:"approved", blockers:[]})), {as_of:url.searchParams.get("as_of"), counts:{distinct_records:1, total_owner_placements:2, matching_owner_placements:2, total_groups:2, matching_groups:2}}),
    /** Comparison inventory declares fingerprints without claiming computed freshness. */
    "/framework-impact/comparisons":()=>s3Page("comparisons", [{comparison_id:s3ComparisonId, resource_id:s3ComparisonId, manifest_sha256:"d".repeat(64), old:s3Fingerprint("res_oldcatalog001"), new:s3Fingerprint("res_newcatalog001", "b"), availability:"valid", diagnostic_code:null, freshness:"not-computed"}]),
    /** Detail binds the unfiltered pair version used by every child page/finding. */
    ["/framework-impact/comparisons/"+s3ComparisonId]:()=>s3ComparisonDetail(),
    /** Exact class filters alter matching rows/counts, preserving complete change counts. */
    ["/framework-impact/comparisons/"+s3ComparisonId+"/changes"]:url=>{
      const items = [{subject_id:"control-1", change_class:"content-changed", old_sha256:"a".repeat(64), new_sha256:"b".repeat(64), old_subjects:[], new_subjects:[], migration:null}];
      const visible = !url.searchParams.has("change_class") || url.searchParams.get("change_class") === "content-changed" ? items : [];
      return s3Page("changes", visible, {resource_version:createHash("sha256").update("changes-"+(url.searchParams.get("change_class")||"all")).digest("hex"), counts:{total_changes:1, matching_changes:visible.length}});
    },
    /** All five exact filters combine with AND while the complete summary remains unchanged. */
    ["/framework-impact/comparisons/"+s3ComparisonId+"/findings"]:url=>{
      const visible = s3Findings().filter(item => (!url.searchParams.has("group") || item.framework_groups.includes(url.searchParams.get("group"))) &&
        (!url.searchParams.has("decision_state") || item.prior_decision_state === url.searchParams.get("decision_state")) &&
        (!url.searchParams.has("policy_source") || item.policy_sources.includes(url.searchParams.get("policy_source"))) &&
        (!url.searchParams.has("priority") || item.priority === url.searchParams.get("priority")) && (!url.searchParams.has("owner") || item.owner === url.searchParams.get("owner")));
      return s3Page("findings", visible, {resource_version:createHash("sha256").update("findings-"+(url.searchParams.get("owner")||"all")).digest("hex")});
    },
    /** Prior-only rows retain declared actor/status metadata without claiming current finding disposition. */
    ["/framework-impact/comparisons/"+s3ComparisonId+"/prior-dispositions"]:()=>s3Page("prior", [{finding_id:"33333333-3333-5333-8333-333333333333", status:"still-open", decided_by:"actor-prior", decided_at:"2026-09-30T00:00:00Z", rationale:"MUST_NOT_RENDER_RATIONALE"}]),
  };
}

/** Unlock and navigate through the complete actual source using the exact additive shell contract. */
async function s3App(overrides = {}) {
  const app = harness({...s3Routes(), ...overrides}, s3Bootstrap);
  await beginUnlock(app).action;
  await app.run('navigate("Lifecycle & Impact")');
  return app;
}

/** Resolve one persistent pager's installed region, live counts and local recovery targets. */
function s3Parts(app, kind) {
  const section = app.byId("view").querySelector(`[data-inspection-page="${kind}"]`);
  assert(section, "Missing S3 pane: "+kind);
  return {section, display:section.querySelector("[data-page-results]"), status:section.querySelector("[data-page-status]"), error:section.querySelector("[data-page-error]")};
}

/** Apply only a caller-authored date through the actual local query form. */
async function s3Date(app, date = "2026-10-02") {
  app.byLabel("Explicit lifecycle review date").value = date;
  await app.byButton("Apply lifecycle date and filters").fire("click");
}

for (const fixture of [{major:1, version:null}, {major:2, version:"2.0.0"}]) {
  /** Preserved old contracts expose their original six destinations and never emit an S3 read. */
  test(`S3 remains unavailable on preserved API${fixture.major} contract ${fixture.version || "1.2.0"}`, async()=>{
    const app = harness({/** Return only the selected old-major Session for the supported old bootstrap. */ "/session/unlock":()=>fixture.major===1?unlockedSession():lifecycleSession()}, fixture.major===1?{"forge-api-major":"1"}:lifecycleBootstrap2);
    await beginUnlock(app).action;
    assert.equal(app.byId("navigation").children.length, 6);
    assert.equal(app.byId("navigation").textContent.includes("Lifecycle & Impact"), false);
    await app.run('navigate("Lifecycle & Impact")');
    assert.match(app.byId("view").textContent, /requires negotiated API2 2.1.0/);
    assert.equal(app.requests.some(request=>/lifecycle|framework-impact/.test(request.route)), false);
  });
}

/** The new contract exposes real read paths, explicit dates, distinct owner scopes and unresolved references. */
test("S3 consumes all nine GET routes without effects, source prose, implicit dates or finding-ID joins", async()=>{
  const app = await s3App();
  assert.equal(app.byId("navigation").children.length, 7);
  assert.equal(app.requests.some(request=>request.route==="/lifecycle/queue"), false);
  assert.equal(app.requests.some(request=>request.route==="/lifecycle/records/"+s3RecordId), false);
  assert.match(s3Parts(app,"records").display.textContent, /Not computed without an explicit date/);
  await app.byButton("Inspect record "+s3RecordId).fire("click");
  assert.match(app.byId("view").textContent, /recorded transition history only/);
  assert.equal(app.requests.some(request=>request.route==="/lifecycle/records/"+s3RecordId), false);
  await s3Date(app);
  assert.match(s3Parts(app,"queue").status.textContent, /1 distinct records · 2 total owner placements.*2 total owner groups/);
  assert.match(s3Parts(app,"queue").display.textContent, /Declared owner owner-A.*Declared owner owner-B/);
  assert.equal(app.requests.some(request=>request.route==="/framework-impact/comparisons/"+s3ComparisonId), false);
  assert.match(app.byId("view").textContent, /Finding identifiers have no comparison pair/);
  await app.byButton("Inspect comparison "+s3ComparisonId).fire("click");
  const expected = ["/lifecycle/records", "/lifecycle/records/"+s3RecordId, "/lifecycle/records/"+s3RecordId+"/history", "/lifecycle/queue",
    "/framework-impact/comparisons", "/framework-impact/comparisons/"+s3ComparisonId, "/framework-impact/comparisons/"+s3ComparisonId+"/changes",
    "/framework-impact/comparisons/"+s3ComparisonId+"/findings", "/framework-impact/comparisons/"+s3ComparisonId+"/prior-dispositions"];
  for (const route of expected) assert(app.requests.some(request=>request.route===route), "Unused route "+route);
  for (const request of app.requests) assert.equal(request.url.startsWith("/api/v2/"), true);
  assert.equal(app.requests.filter(request=>/lifecycle|framework-impact/.test(request.route)).every(request=>request.options.method==="GET" && request.options.body===undefined), true);
  assert.equal(app.requests.some(request=>/effects|exports|operations|resources\/register/.test(request.route)), false);
  assert.equal(app.byId("view").textContent.includes("MUST_NOT_RENDER"), false);
  assert.match(s3Parts(app,"findings").status.textContent, /2 complete findings · 2 matching findings/);
  assert.match(s3Parts(app,"prior").status.textContent, /1 prior-only dispositions/);
});

/** Applied filters use their own page version and same-response full counts; no forbidden denominator query is made. */
test("S3 changes and all five finding filters preserve the complete unfiltered summary", async()=>{
  const app=await s3App(); await app.byButton("Inspect comparison "+s3ComparisonId).fire("click");
  const initialRequests=app.requests.length;
  app.byLabel("Impact change class").value="added";
  await app.byButton("Apply Framework impact changes filters").fire("click");
  assert.match(s3Parts(app,"changes").status.textContent,/1 complete changes · 0 matching changes/);
  app.byLabel("Impact framework group").value="group-A";
  app.byLabel("Impact prior decision state").value="applicable";
  app.byLabel("Impact policy source identity").value="source-A";
  app.byLabel("Impact finding priority").value="blocking";
  app.byLabel("Impact owner key").value="owner-A";
  await app.byButton("Apply Framework impact findings filters").fire("click");
  const calls=app.requests.slice(initialRequests);
  assert.equal(calls.length,2);
  const url=new URL(calls[1].url,"http://127.0.0.1:1");
  assert.deepEqual([...url.searchParams.keys()], ["page_size","group","decision_state","policy_source","priority","owner"]);
  assert.match(s3Parts(app,"findings").status.textContent,/2 complete findings · 1 matching findings/);
  assert.match(s3Parts(app,"findings").display.textContent,/Complete unfiltered comparison summary/);
  assert.match(s3Parts(app,"findings").display.textContent,/Filtered emitted disposition scope/);
  assert.equal(s3Parts(app,"findings").display.textContent.includes(s3OtherFindingId),false);
  assert.equal(app.run("dirty"),false);
});

/** Opaque cursor reuse stays on one page/capture; a changed capture preserves verified rows and local restart recovery. */
test("S3 page two uses its retained cursor and rejects changed capture without replacing counts or rows",async()=>{
  let changed=false;
  const app=await s3App({/** Supply 50 then one record under a 51-item page chain, with an optional capture conflict. */ "/lifecycle/records":url=>{
    const second=url.searchParams.has("cursor");
    const items=second?[s3Record("res_record000051")]:Array.from({length:50},(_,n)=>s3Record("res_record"+String(n+1).padStart(6,"0")));
    return s3Page("records",items,{snapshot_version:changed?"d".repeat(64):s3Snapshot, page:{items,total_matching:51,next_cursor:second?null:"opaque-record-page2"}, counts:{registered_records:51,matching_records:51,unavailable_records:0}});
  }});
  const first=s3Parts(app,"records");
  await app.byButton("Next Lifecycle record inventory page").fire("click");
  assert.match(first.display.textContent,/res_record000051/);
  assert.match(first.status.textContent,/Page 2/);
  await app.byButton("Previous Lifecycle record inventory page").fire("click");
  const retained=first.display.textContent; const prior=first.status.textContent;
  changed=true; await app.byButton("Next Lifecycle record inventory page").fire("click");
  assert.equal(first.display.textContent,retained);
  assert.match(first.error.textContent,/page chain changed/);
  assert.match(first.status.textContent,/Previous capture/);
  assert.equal(app.document.activeElement===first.error,true,"Capture error must own focus");
  assert.equal(app.byButton("Restart Lifecycle record inventory from first page").hidden,false);
  changed=false;await app.byButton("Restart Lifecycle record inventory from first page").fire("click");
  assert.match(first.status.textContent,/Page 1/);
  assert.equal(first.error.hidden,true);
  assert(prior.includes("51 registered records"));
});

/** A dated request from an obsolete installed view cannot replace a later date's capture or focus. */
test("S3 navigation and a newer explicit date fence the older inventory and queue",async()=>{
  const old=deferred();let hold=false;
  const app=await s3App({/** Hold only the first dated inventory while a later installed view/date completes normally. */ "/lifecycle/records":url=>hold&&url.searchParams.get("as_of")==="2026-10-02"?old.promise:s3Page("records",[s3Record(s3RecordId,url.searchParams.get("as_of"))],{as_of:url.searchParams.get("as_of")})});
  hold=true;app.byLabel("Explicit lifecycle review date").value="2026-10-02";
  const first=app.byButton("Apply lifecycle date and filters").fire("click");await settle();
  await app.run('navigate("Overview")');await app.run('navigate("Lifecycle & Impact")');
  await s3Date(app,"2026-10-03");
  const text=app.byId("view").textContent;const focus=app.document.activeElement;const global=app.byId("status").textContent;
  old.resolve(s3Page("records",[s3Record(s3RecordId,"2026-10-02")],{as_of:"2026-10-02"}));await first;
  assert.equal(app.byId("view").textContent,text);assert.equal(app.byId("status").textContent,global);
  assert.equal(app.document.activeElement===focus,true,"Older date read must not take newer focus");
  const dated=app.requests.filter(request=>request.route==="/lifecycle/queue");
  assert.equal(new URL(dated.at(-1).url,"http://127.0.0.1:1").searchParams.get("as_of"),"2026-10-03");
});

/** Pending Apply keeps its original date/filter tuple and ignores repeated activation until it settles. */
test("S3 pending lifecycle Apply preserves its captured date and blocks duplicate reads",async()=>{
  const held=deferred();const app=await s3App({/** Hold one dated inventory while the query form can be edited without altering the sent request. */ "/lifecycle/records":url=>url.searchParams.has("as_of")?held.promise:s3Page("records",[s3Record()])});
  const apply=app.byButton("Apply lifecycle date and filters");app.byLabel("Explicit lifecycle review date").value="2026-10-02";apply.focus();
  const pending=apply.fire("click");await settle();app.byLabel("Explicit lifecycle review date").value="2026-10-03";await apply.fire("click");
  assert.equal(app.document.activeElement===apply,true,"Pending query must retain its focusable trigger");
  assert.equal(app.requests.filter(request=>request.route==="/lifecycle/records"&&new URL(request.url,"http://127.0.0.1:1").searchParams.has("as_of")).length,1);
  held.resolve(s3Page("records",[s3Record(s3RecordId,"2026-10-02")],{as_of:"2026-10-02"}));await pending;
  const queue=app.requests.filter(request=>request.route==="/lifecycle/queue").at(-1);
  assert.equal(new URL(queue.url,"http://127.0.0.1:1").searchParams.get("as_of"),"2026-10-02");
  assert.equal(apply.getAttribute("aria-disabled"),"false");
});

/** Cross-endpoint captures and finding pair context must both match the selected detail before any child group installs. */
for (const defect of ["snapshot", "comparison", "version", "full-summary", "emitted-counts"]) {
  /** Reject this concrete capture, pair, full-summary or emitted-count defect before group installation. */
  test("S3 rejects staged comparison " + defect + " without installing a mixed-context finding",async()=>{
    const app=await s3App({/** Return one deliberate peer-context defect while leaving the other two subpages authentic to the fixture. */ ["/framework-impact/comparisons/"+s3ComparisonId+"/findings"]:()=>{
      const result=s3Page("findings",s3Findings());
      if(defect==="snapshot")result.snapshot_version="d".repeat(64);
      if(defect==="comparison")result.page.items[0].comparison_id="res_foreign01";
      if(defect==="version")result.page.items[0].comparison_version="d".repeat(64);
      if(defect==="full-summary")result.full_summary.blocking=99;
      if(defect==="emitted-counts")result.emitted_dispositions.undispositioned=99;
      return result;
    }});
    await app.byButton("Inspect comparison "+s3ComparisonId).fire("click");
    const selected=app.byId("view").querySelector('[data-inspection-selection="Selected framework impact comparison"]');
    assert.equal(selected.querySelector("[data-page-error]").hidden,false);
    assert.equal(selected.textContent.includes("Finding "+s3FindingId),false);
    assert.equal(selected.querySelector('[data-inspection-page="findings"]'),null);
    assert.equal(app.document.activeElement===selected.querySelector("[data-page-error]"),true,"Selected capture error must keep local focus");
  });
}

/** A typed service failure stays local and retries one captured selection, preserving global and dirty states. */
test("S3 typed query-budget failure keeps local recovery and does not announce Saved",async()=>{
  let fails=true;
  const app=await s3App({/** Fail only selected detail, then allow its exact explicit retry to stage all children. */ ["/framework-impact/comparisons/"+s3ComparisonId]:()=>fails?{status:503,body:{code:"query-budget-exceeded",message:"The captured query budget was exhausted.",retryable:true}}:s3ComparisonDetail()});
  const global=app.byId("status").textContent;
  await app.byButton("Inspect comparison "+s3ComparisonId).fire("click");
  const selected=app.byId("view").querySelector('[data-inspection-selection="Selected framework impact comparison"]');
  assert.match(selected.querySelector("[data-page-error]").textContent,/query-budget-exceeded/);
  assert.equal(app.byId("status").textContent,global);
  assert.equal(app.run("dirty"),false);
  assert.equal(app.byId("status").textContent.includes("Saved"),false);
  fails=false;await app.byButton("Retry Selected framework impact comparison").fire("click");
  assert.equal(selected.querySelector("[data-page-error]").hidden,true);
  assert.match(s3Parts(app,"findings").display.textContent,new RegExp(s3FindingId));
});

/** Dirty cancellation keeps the connected trigger and prevents a new inspection request. */
test("S3 Keep editing leaves existing metadata and caller focus without sending a selected read",async()=>{
  const app=await s3App();app.run("dirty=true");const trigger=app.byButton("Inspect comparison "+s3ComparisonId);trigger.focus();
  const count=app.requests.length;const action=trigger.fire("click");await settle();
  await app.byButton("Keep editing").fire("click");await action;
  assert.equal(app.requests.length,count);
  assert.equal(app.run("dirty"),true);
  assert.equal(app.document.activeElement===trigger,true,"Keep editing must preserve the initiating connected trigger");
});

for(const transition of ["navigate","shutdown"]) {
  /** A late selected detail cannot publish metadata/error/focus over a newer view or stopped session. */
  test(`S3 late selected comparison after ${transition} is retired without child queries`,async()=>{
    const held=deferred();const app=await s3App({/** Hold actual selected detail until a newer UI lifecycle owns the page. */ ["/framework-impact/comparisons/"+s3ComparisonId]:()=>held.promise,
      /** Acknowledge normal selected-major shutdown so stopped fences can be observed. */ "/session/shutdown":()=>({state:"shutting-down"})});
    const action=app.byButton("Inspect comparison "+s3ComparisonId).fire("click");await settle();
    if(transition==="navigate")await app.run('navigate("Overview")');
    else {await app.byId("stop").fire("click");await app.byId("confirm-stop").fire("click");}
    const focus=app.document.activeElement;const text=app.byId("view").textContent;const status=app.byId("status").textContent;
    held.resolve(s3ComparisonDetail());await action;
    assert.equal(app.byId("view").textContent,text);assert.equal(app.byId("status").textContent,status);
    assert.equal(app.document.activeElement===focus,true,"Late selection must not steal newer focus");
    assert.equal(app.requests.some(request=>/\/changes$|\/findings$|\/prior-dispositions$/.test(request.route)),false);
  });
}

/** Owner filtering preserves full distinct membership/placements/groups and sends no queue state filter. */
test("S3 lifecycle owner filter keeps distinct full-scope and matching placement counts separate",async()=>{
  const app=await s3App({/** Apply the exact authored owner to rows while preserving unfiltered queue counts. */ "/lifecycle/queue":url=>s3Page("queue",[{record_id:s3RecordId,resource_id:s3RecordId,owner_key:url.searchParams.get("owner"),next_review_date:"2026-10-03",policy_key:"policy-key",version_key:"v1",state:"approved",derived_status:"approved",blockers:[]}],{as_of:url.searchParams.get("as_of"),counts:{distinct_records:1,total_owner_placements:2,matching_owner_placements:1,total_groups:2,matching_groups:1}})});
  app.byLabel("Lifecycle owner key").value="owner-A";app.byLabel("Stored lifecycle state").value="approved";await s3Date(app);
  const records=new URL(app.requests.filter(request=>request.route==="/lifecycle/records").at(-1).url,"http://127.0.0.1:1");
  const queue=new URL(app.requests.filter(request=>request.route==="/lifecycle/queue").at(-1).url,"http://127.0.0.1:1");
  assert.equal(records.searchParams.get("owner"),"owner-A");assert.equal(records.searchParams.get("state"),"approved");
  assert.equal(queue.searchParams.get("owner"),"owner-A");assert.equal(queue.searchParams.has("state"),false);
  assert.match(s3Parts(app,"queue").status.textContent,/1 distinct records · 2 total owner placements · 1 matching owner placements · 2 total owner groups · 1 matching owner groups/);
  assert.equal(s3Parts(app,"queue").display.textContent.includes("owner-B"),false);
});

/** Failed cross-endpoint date staging keeps the old inventory's usable recorded-history controls and date semantics. */
test("S3 mixed inventory/queue captures retain the prior date and do not compute a selected record",async()=>{
  const app=await s3App({/** Supply a foreign queue capture so the dated group cannot replace recorded inventory. */ "/lifecycle/queue":url=>s3Page("queue",[],{snapshot_version:"d".repeat(64),as_of:url.searchParams.get("as_of"),counts:{distinct_records:0,total_owner_placements:0,matching_owner_placements:0,total_groups:0,matching_groups:0}})});
  const before=s3Parts(app,"records").display.textContent;await s3Date(app);
  assert.equal(s3Parts(app,"records").display.textContent,before);
  assert.match(app.byId("view").textContent,/could not be staged at one capture/);
  await app.byButton("Inspect record "+s3RecordId).fire("click");
  assert.equal(app.requests.some(request=>request.route==="/lifecycle/records/"+s3RecordId),false);
  assert.match(app.byId("view").textContent,/recorded transition history only/);
});

for(const availability of ["absent-index","index-upgrade-required","empty"]) {
  /** Zero counts retain distinct setup states and never infer approval or query a date-dependent queue. */
  test(`S3 zero inventory preserves ${availability} separately`,async()=>{
    const app=await s3App({/** Supply a closed zero record envelope with its actual setup state. */ "/lifecycle/records":()=>s3Page("records",[],{availability}),
      /** Supply the same explicit comparison setup state independently. */ "/framework-impact/comparisons":()=>s3Page("comparisons",[],{availability})});
    assert.match(s3Parts(app,"records").status.textContent,new RegExp(availability+": 0 registered records"));
    assert.match(s3Parts(app,"comparisons").status.textContent,new RegExp(availability+": 0 registered comparisons"));
    assert.equal(app.requests.some(request=>request.route==="/lifecycle/queue"),false);
  });
}

/** An explicit newer focus target is retained when a still-owned earlier detail read completes. */
test("S3 pending comparison success does not take focus from a later lifecycle date control",async()=>{
  const held=deferred();const app=await s3App({/** Hold selected detail while the user deliberately moves focus to an independent query lane. */ ["/framework-impact/comparisons/"+s3ComparisonId]:()=>held.promise});
  const trigger=app.byButton("Inspect comparison "+s3ComparisonId);trigger.focus();const action=trigger.fire("click");await settle();
  const target=app.byLabel("Explicit lifecycle review date");target.focus();held.resolve(s3ComparisonDetail());await action;
  assert.equal(app.document.activeElement===target,true,"A later connected focus target must retain focus");
  assert.match(s3Parts(app,"findings").display.textContent,new RegExp(s3FindingId));
});

/** A finding page keeps complete filtered counts/dispositions even when only 50 rows are displayed. */
test("S3 finding pagination retains whole filtered counts and exact comparison context across page two",async()=>{
  const summary=s3Summary({findings:51,blocking:0,review_required:51,dispositioned_resolved:0,undispositioned:51});
  const rows=Array.from({length:51},(_,index)=>({...s3Findings()[1],finding_id:"00000000-0000-5000-8000-"+String(index+1).padStart(12,"0")}));
  const app=await s3App({/** Bind complete unfiltered counts to the fixed comparison detail. */ ["/framework-impact/comparisons/"+s3ComparisonId]:()=>s3ComparisonDetail({summary}),
    /** Return only bounded selected rows, but retain all 51 filtered findings/dispositions in page metadata. */ ["/framework-impact/comparisons/"+s3ComparisonId+"/findings"]:url=>{
      const second=url.searchParams.has("cursor");const items=second?rows.slice(50):rows.slice(0,50);
      return s3Page("findings",items,{page:{items,total_matching:51,next_cursor:second?null:"opaque-finding-page2"},counts:{total_findings:51,matching_findings:51},full_summary:summary,emitted_dispositions:{resolved:0,accepted_risk:0,still_open:0,undispositioned:51}});
    }});
  await app.byButton("Inspect comparison "+s3ComparisonId).fire("click");
  const pane=s3Parts(app,"findings");assert.match(pane.status.textContent,/51 complete findings · 51 matching findings.*Page 1/);
  await app.byButton("Next Framework impact findings page").fire("click");
  assert.match(pane.status.textContent,/51 complete findings · 51 matching findings.*Page 2/);
  assert.equal(pane.display.textContent.includes(rows[50].finding_id),true);assert.equal(pane.display.textContent.includes(rows[0].finding_id),false);
  const reads=app.requests.filter(request=>request.route.endsWith("/findings"));assert.equal(reads.length,2);
  assert.equal(new URL(reads[1].url,"http://127.0.0.1:1").searchParams.get("cursor"),"opaque-finding-page2");
  assert.equal(pane.error.hidden,true);
});

/** Handled filtered-page errors and failed retries keep the owned local alert rather than the invoking button. */
test("S3 handled page API failure and retry retain local error focus and verified rows", async()=>{
  let fails=true;
  const app=await s3App({/** Fail only the selected changes filter; initial complete comparison remains valid. */ ["/framework-impact/comparisons/"+s3ComparisonId+"/changes"]:url=>url.searchParams.has("change_class")&&fails?{status:503,body:{code:"query-budget-exceeded",message:"The captured query budget was exhausted.",retryable:true}}:s3Page("changes",[{subject_id:"ac-1",change_class:"content-changed",old_sha256:"a".repeat(64),new_sha256:"b".repeat(64),old_subjects:[],new_subjects:[],migration:null}])});
  await app.byButton("Inspect comparison "+s3ComparisonId).fire("click");
  const pane=s3Parts(app,"changes");const rows=pane.display.textContent;const global=app.byId("status").textContent;
  app.byLabel("Impact change class").value="content-changed";
  const trigger=app.byButton("Apply Framework impact changes filters");trigger.focus();await trigger.fire("click");
  assert.equal(pane.display.textContent,rows);assert.match(pane.error.textContent,/query-budget-exceeded/);
  assert.equal(app.document.activeElement===pane.error,true,"Handled page failure must keep the local alert focused");
  const retry=app.byButton("Retry Framework impact changes");retry.focus();await retry.fire("click");
  assert.equal(app.document.activeElement===pane.error,true,"Failed page retry must not refocus its trigger");
  const query=new URL(app.requests.filter(request=>request.route.endsWith("/changes")).at(-1).url,"http://127.0.0.1:1");
  assert.equal(query.searchParams.get("change_class"),"content-changed");
  fails=false;retry.focus();await retry.fire("click");assert.equal(pane.error.hidden,true);
  assert.equal(app.byId("status").textContent,global);assert.equal(app.run("dirty"),false);
});

/** A selected dated record's ordinary API failure and retry remain local, never requesting history on failed detail. */
test("S3 handled selected record failure and retry retain local alert focus",async()=>{
  let fails=true;
  const app=await s3App({/** Reject only selected record detail, with a normal typed not-found response. */ ["/lifecycle/records/"+s3RecordId]:url=>fails?{status:404,body:{code:"not-found",message:"The registered record is unavailable.",retryable:false}}:s3RecordDetail(url)});
  await s3Date(app);const trigger=app.byButton("Inspect record "+s3RecordId);trigger.focus();await trigger.fire("click");
  const selected=app.byId("view").querySelector('[data-inspection-selection="Selected lifecycle record"]');const error=selected.querySelector("[data-page-error]");
  assert.match(error.textContent,/not-found/);assert.equal(app.document.activeElement===error,true,"Selected record error must remain focused");
  assert.equal(app.requests.some(request=>request.route.endsWith("/history")),false);
  const retry=app.byButton("Retry Selected lifecycle record");retry.focus();await retry.fire("click");
  assert.equal(app.document.activeElement===error,true,"Selected record retry must preserve local error focus");
  fails=false;await retry.fire("click");assert.equal(error.hidden,true);
  assert.equal(selected.querySelector('[data-inspection-page="history"]')!==null,true);
});

/** A comparison detail budget failure and failed selection retry preserve the same owned alert. */
test("S3 handled selected comparison failure and retry retain local alert focus",async()=>{
  const app=await s3App({/** Return a typed query budget failure before any comparison child can be staged. */ ["/framework-impact/comparisons/"+s3ComparisonId]:()=>({status:503,body:{code:"query-budget-exceeded",message:"The captured query budget was exhausted.",retryable:true}})});
  const trigger=app.byButton("Inspect comparison "+s3ComparisonId);trigger.focus();await trigger.fire("click");
  const selected=app.byId("view").querySelector('[data-inspection-selection="Selected framework impact comparison"]');const error=selected.querySelector("[data-page-error]");
  assert.equal(app.document.activeElement===error,true,"Comparison detail failure must keep the local alert focused");
  const retry=app.byButton("Retry Selected framework impact comparison");retry.focus();await retry.fire("click");
  assert.equal(app.document.activeElement===error,true,"Failed comparison retry must not refocus its trigger");
  assert.equal(app.requests.some(request=>/\/changes$|\/findings$|\/prior-dispositions$/.test(request.route)),false);
});

/** Local date validation keeps its alert focused and sends no query or replacement capture. */
test("S3 invalid explicit date retains local alert focus without invoking a read",async()=>{
  const app=await s3App();const prior=s3Parts(app,"records").display.textContent;const count=app.requests.length;
  app.byLabel("Explicit lifecycle review date").value="2026-2-02";
  const trigger=app.byButton("Apply lifecycle date and filters");trigger.focus();await trigger.fire("click");
  const error=app.byId("view").querySelectorAll('[role="alert"]').find(item=>!item.hidden);
  assert(error);assert.match(error.textContent,/explicit valid YYYY-MM-DD/);
  assert.equal(app.document.activeElement===error,true,"Local date validation must not refocus Apply");
  assert.equal(app.requests.length,count);assert.equal(s3Parts(app,"records").display.textContent,prior);
});

/** A failed dated inventory/queue staging retains the old date scope and its local error focus. */
test("S3 handled lifecycle Apply failure retains local alert focus and prior date scope",async()=>{
  const app=await s3App({/** Fail only explicitly dated inventory; the initial no-date inventory remains usable. */ "/lifecycle/records":url=>url.searchParams.has("as_of")?{status:503,body:{code:"query-budget-exceeded",message:"The captured query budget was exhausted.",retryable:true}}:s3Page("records",[s3Record()])});
  const prior=s3Parts(app,"records").display.textContent;
  app.byLabel("Explicit lifecycle review date").value="2026-10-02";
  const trigger=app.byButton("Apply lifecycle date and filters");trigger.focus();await trigger.fire("click");
  const error=app.byId("view").querySelectorAll('[role="alert"]').find(item=>!item.hidden);
  assert(error);assert.match(error.textContent,/Previous date\/filter results are retained/);
  assert.equal(app.document.activeElement===error,true,"Failed lifecycle staging must preserve its local alert focus");
  assert.equal(s3Parts(app,"records").display.textContent,prior);
  await app.byButton("Inspect record "+s3RecordId).fire("click");
  assert.equal(app.requests.some(request=>request.route==="/lifecycle/records/"+s3RecordId),false);
  assert.equal(app.byId("view").textContent.includes("recorded transition history only"),true);
});


/** Construct a complete synthetic index replacement; these rows establish UI facts, not native admission. */
function s6Replacement(previous = metadataPreview(1).bundle.index, targetVersion = 1) {
  const proposed = {schema_version:`forge.workspace/${targetVersion}`,label:"Replacement <script> π",resources:[{key:"new-key",role:"policy-source",path:"new.md"}]};
  const hash = createHash("sha256").update(JSON.stringify(proposed,null,2)+"\n").digest("hex");
  const preview = {...proposedWrite(),operation_type:"workspace-index-update",target:{status:previous ? "overwrite" : "create",path:"forge.workspace.json"},exact_bytes_sha256:hash};
  return {validation:{state:"valid",error_count:0,warning_count:0,diagnostics:[]},preview,replacement:{previous_index:previous,proposed_index:proposed,
    supplied_index_sha256:hash,proposed_index_sha256:hash,removed_resource_keys:previous ? previous.resources.filter(row=>row.key!=="new-key").map(row=>row.key) : [],consumed_file_count:previous ? previous.resources.length+2 : 1}};
}

/** Unlock and install the actual 2.2 forms while allowing only authored transport observations. */
async function s6App(overrides = {}, readOnly = false, version = "2.2.0") {
  const reply=s6Replacement();
  const app=harness({
    /** Match the explicit shell contract through the unmodified unlock handler. */
    "/session/unlock":()=>lifecycleSession(readOnly,{contract_version:version}),
    /** Provide a closed, complete synthetic direct import projection. */
    "/project/bundle-imports":()=>reply,
    /** Reload the exact synthetic receipt by its prepared identity. */
    ["/effects/previews/"+reply.preview.preview_id]:()=>reply.preview,
    /** A cancelled synthetic export leaves no publishable result. */
    "/project/bundle-exports":()=>operation("cancelled",{kind:"export"}),
    ...overrides,
  },{"forge-api-major":"2","forge-api-contract-version":version});
  await beginUnlock(app).action;await app.run('navigate("Trace & Reports")');return app;
}

/** Resolve the new form independently from the old local-preview/comparison metadata panel. */
function s6Parts(app) {
  const panel=app.byId("view").querySelector("[data-bundle-effects]");assert(panel);
  return {panel,target:app.byLabel("Metadata export project-relative target"),sensitive:app.byLabel("I acknowledge that exported labels, keys, paths and hashes are sensitive metadata."),
    file:app.byLabel("Choose a bundle for index replacement"),schema:app.byLabel("Replacement target index schema"),ack:app.byLabel("I acknowledge replacing the index label and all registrations; source files will not be deleted."),
    export:app.byButton("Prepare metadata export",panel),import:app.byButton("Prepare index replacement",panel),error:panel.querySelector('[role="alert"]')};
}

/** Select exact opaque bytes and acknowledge after the chosen file has invalidated prior acknowledgment. */
async function s6Choose(app, bytes=Buffer.from('{"schema_version":"synthetic"}'), version=1, read) {
  const parts=s6Parts(app);const buffer=Buffer.from(bytes);
  parts.file.files=[{name:"chosen-bundle.json",size:buffer.length,
    /** Return the original binary slice, or a deliberately controlled asynchronous File read. */
    async arrayBuffer(){return read ? read() : buffer.buffer.slice(buffer.byteOffset,buffer.byteOffset+buffer.byteLength);}}];
  await parts.file.fire("change");parts.schema.value=String(version);await parts.schema.fire("change");parts.ack.checked=true;await parts.ack.fire("change");return parts;
}

/** Author a sensitive destination acknowledgment only after the actual input-change handler resets it. */
async function s6ExportReady(app) {
  const parts=s6Parts(app);parts.target.value="exports/metadata.json";await parts.target.fire("input");parts.sensitive.checked=true;await parts.sensitive.fire("change");return parts;
}

/** Supply the actual committed result envelope for the same exact target as its prepared receipt. */
function s6Committed(preview) {
  return operation("succeeded",{kind:"commit",result:{target_path:preview.target.path,committed_sha256:preview.exact_bytes_sha256,new_version:"synthetic-committed-version",write_committed:true}});
}

/** Prepare, confirm and expose the actual metadata download control with an explicitly authored JSON artifact. */
async function s6DownloadApp(downloadOverride) {
  const bytes=Buffer.from(JSON.stringify(metadataPreview(1).bundle));
  const preview={...proposedWrite(),operation_type:"report-export",target:{status:"create",path:"exports/metadata.json"},exact_bytes_sha256:createHash("sha256").update(bytes).digest("hex")};
  const app=await s6App({
    /** A terminal export response retains only the prepared effect until explicit confirmation. */
    "/project/bundle-exports":()=>operation("succeeded",{kind:"export",result:{preview}}),
    /** Reload the same synthetic artifact receipt before review. */
    ["/effects/previews/"+preview.preview_id]:()=>preview,
    /** Acknowledge one exact synthetic commit operation. */
    "/effects/commits":()=>operation("pending",{kind:"commit"}),
    /** Return only the committed target identity required by the existing saved-result guards. */
    [operationRoute]:()=>s6Committed(preview),
    /** Supply exact artifact bytes and typed media, or an adversarial authored response. */
    ["/project/bundle-exports/"+operationId+"/download"]:downloadOverride ?? (()=>({download_blob:new Blob([bytes]),download_media_type:"application/json; charset=utf-8"})),
  });
  const parts=await s6ExportReady(app);parts.export.focus();await parts.export.fire("click");
  assert.equal(app.byId("preview-dialog").open,true);await app.byButton("Confirm this exact write",app.byId("preview-dialog")).fire("click");
  return {app,bytes,preview,download:app.byButton("Download committed metadata bundle")};
}

// These are appended actual-asset source controls. The authored transport is not native/API admission evidence.
for (const version of ["2.0.0","2.1.0","2.2.0"]) {
  test("S6 effect forms require exact2.2 while S3 navigation remains available at2.1/2.2: "+version,async()=>{
    const app=await s6App({},false,version);
    assert.equal(!!app.byId("view").querySelector("[data-bundle-effects]"),version==="2.2.0");
    assert.equal(app.run('titles.includes("Lifecycle & Impact")'),version!=="2.0.0");
    assert.equal(app.requests.some(request=>["/project/bundle-imports","/project/bundle-exports"].includes(request.route)),false);
  });
}

test("S6 readonly forms retain query access and reject both prepare controls without dispatch",async()=>{
  const app=await s6App({},true);const parts=await s6Choose(app);parts.target.value="metadata.json";parts.sensitive.checked=true;await parts.sensitive.fire("change");
  assert.equal(parts.import.getAttribute("aria-disabled"),"true");assert.equal(parts.export.getAttribute("aria-disabled"),"true");
  await parts.import.fire("click");await parts.export.fire("click");
  assert.equal(app.requests.some(request=>["/project/bundle-imports","/project/bundle-exports"].includes(request.route)),false);
  assert.equal(metadataParts(app).preview.isConnected,true);assert.match(parts.panel.textContent,/Read-only session/);
});

test("S6 raw numeric envelope preserves BOM, Unicode and duplicate keys while keeping old verification overhead11",async()=>{
  const app=await s6App();const raw=Buffer.from('\ufeff{"key":"π","key":"unchanged duplicate"}');const parts=await s6Choose(app,raw,2);
  await parts.import.fire("click");const sent=app.requests.find(request=>request.route==="/project/bundle-imports");assert(sent);
  const actual=Buffer.from(await sent.options.body.arrayBuffer());const prefix=Buffer.from('{"bundle":');const suffix=Buffer.from(',"target_index_schema_version":2,"acknowledge_index_replacement":true}');
  assert.equal(prefix.length+suffix.length,80);assert.deepEqual(actual,Buffer.concat([prefix,raw,suffix]));assert.match(sent.options.headers["Idempotency-Key"],/^[a-f0-9-]{36}$/);
  const verify=await app.run('bundleVerificationBody({size:2,arrayBuffer:async()=>new Uint8Array([123,125]).buffer})');assert.equal(verify.size,13);
});

for (const extra of [0,1]) {
  test("S6 full import body boundary "+(extra ? "rejects one excess byte before read" : "allows exact1MiB"),async()=>{
    const app=await s6App();app.run(`globalThis.s6Reads=0;globalThis.s6File={size:${1048496+extra},arrayBuffer:async()=>{s6Reads++;return new Uint8Array(${1048496+extra}).buffer;}}`);
    if(extra){await assert.rejects(app.run('bundleImportBody(s6File,1)'),/80-byte/);assert.equal(app.run("s6Reads"),0);}
    else {const body=await app.run('bundleImportBody(s6File,1)');assert.equal(body.size,1048576);assert.equal(app.run("s6Reads"),1);}
  });
}

test("S6 changing file during asynchronous read retires old bytes and keeps newer form available",async()=>{
  const app=await s6App();const reading=deferred();const parts=await s6Choose(app,Buffer.from("{}"),1,()=>reading.promise);parts.import.focus();const action=parts.import.fire("click");await settle();
  await s6Choose(app,Buffer.from("{\"new\":true}"));assert.equal(parts.import.getAttribute("aria-disabled"),"false");
  reading.resolve(new Uint8Array([123,125]).buffer);await action;assert.equal(app.requests.some(request=>request.route==="/project/bundle-imports"),false);assert.equal(parts.panel.getAttribute("aria-busy"),"false");
});

test("S6 destination editing resets sensitivity acknowledgment and preserves prepared-operation recovery",async()=>{
  const delayed=deferred();const app=await s6App({"/project/bundle-exports":()=>delayed.promise});const parts=await s6ExportReady(app);parts.export.focus();const action=parts.export.fire("click");await settle();
  parts.target.value="changed.json";await parts.target.fire("input");assert.equal(parts.sensitive.checked,false);assert.equal(parts.panel.getAttribute("aria-busy"),"false");
  delayed.resolve(operation("succeeded",{kind:"export",result:{preview:proposedWrite()}}));await action;
  assert.equal(app.byId("preview-dialog").open,false);assert.equal(operationParts(app).row.isConnected,true);
  assert.equal(app.requests.filter(request=>request.route==="/project/bundle-exports").length,1);
});

test("S6 unknown import outcome retries identical raw bytes and key even after file selection changes",async()=>{
  let attempts=0;const delayed=deferred();const reply=s6Replacement();const app=await s6App({"/project/bundle-imports":()=>++attempts===1?delayed.promise:reply});
  const parts=await s6Choose(app,Buffer.from("{}"));parts.import.focus();const action=parts.import.fire("click");await settle();await s6Choose(app,Buffer.from('{"changed":true}'));
  delayed.resolve({status:503,body:{code:"query-budget-exceeded",message:"Synthetic deadline",retryable:true}});await action;
  const focus=app.document.activeElement;const recovery=requestParts(app);assert.equal(app.byId("error").hidden,true);assert.equal(app.document.activeElement===focus,true);
  await app.byButton("Retry the same request",recovery.row).fire("click");const sent=app.requests.filter(request=>request.route==="/project/bundle-imports");assert.equal(sent.length,2);
  assert.equal(sent[0].options.headers["Idempotency-Key"],sent[1].options.headers["Idempotency-Key"]);assert.equal(sent[0].options.body===sent[1].options.body,true);assert.equal(app.byId("preview-dialog").open,false);
  const review=app.byButton("Review prepared write",recovery.row).fire("click");await settle();assert.equal(app.document.querySelector("dialog[open]").getAttribute("aria-labelledby"),"discard-title");await app.byButton("Discard edits").fire("click");await review;assert.equal(app.byId("preview-dialog").open,true);
});

for (const previous of [null,{schema_version:"forge.workspace/1",label:"Explicitly empty",resources:[]},metadataPreview(3).bundle.index]) {
  test("S6 replacement retains complete membership and distinguishes "+(previous===null?"absence":previous.resources.length+" prior registrations"),async()=>{
    const reply=s6Replacement(previous);const app=await s6App({"/project/bundle-imports":()=>reply,["/effects/previews/"+reply.preview.preview_id]:()=>reply.preview});const parts=await s6Choose(app);parts.import.focus();await parts.import.fire("click");
    const content=app.byId("preview-content");assert.equal(app.byId("preview-dialog").open,true);assert.match(content.textContent,/Complete project index replacement/);assert.match(content.textContent,/Supplied normalized index SHA-256/);
    assert.match(content.textContent,previous===null?/No project index was present/:new RegExp(previous.resources.length+" registrations"));
    assert.equal(content.querySelectorAll("tbody").reduce((sum,body)=>sum+body.children.length,0),(previous?.resources.length??0)+1);
    await app.byButton("Keep editing",app.byId("preview-dialog")).fire("click");assert.equal(app.document.activeElement===parts.import,true);assert.equal(app.requests.some(request=>request.route==="/effects/commits"),false);
  });
}

for (const defect of ["wrong-target","wrong-hash","dropped-removed-key","unreconciled-warnings","wrong-complete-membership","undercounted-path-union","inflated-path-union"]) {
  test("S6 rejects inconsistent replacement projection before opening receipt: "+defect,async()=>{
    const reply=s6Replacement();if(defect==="wrong-target")reply.replacement.proposed_index.schema_version="forge.workspace/2";
    if(defect==="wrong-hash")reply.replacement.proposed_index_sha256="b".repeat(64);
    if(defect==="dropped-removed-key")reply.replacement.removed_resource_keys=[];
    if(defect==="unreconciled-warnings")reply.validation.warning_count=1;
    if(defect==="wrong-complete-membership")reply.replacement.proposed_index.resources[0].path="other.md";
    if(defect==="undercounted-path-union")reply.replacement.consumed_file_count=2;
    if(defect==="inflated-path-union")reply.replacement.consumed_file_count=4;
    const app=await s6App({"/project/bundle-imports":()=>reply});const parts=await s6Choose(app);await parts.import.fire("click");
    assert.equal(app.byId("preview-dialog").open,false);assert.equal(app.requests.some(request=>request.route.startsWith("/effects/previews/") || request.route==="/effects/commits"),false);assert.equal(requestParts(app).row.isConnected,true);
  });
}

test("S6 committed JSON download binds authenticated route, media, hash and fixed filename without replay",async()=>{
  const {app,bytes,download}=await s6DownloadApp();download.focus();await download.fire("click");
  const sent=app.requests.find(request=>request.route.endsWith("/download"));assert.equal(sent.route,"/project/bundle-exports/"+operationId+"/download");assert.equal(sent.options.credentials,"omit");assert.equal(sent.options.redirect,"error");
  assert.equal(app.document.activeElement===download,true);assert.equal(app.document.downloads.length,1);const saved=app.document.downloads[0];assert.equal(saved.filename,"forge-workspace-index-and-hashes.json");assert.deepEqual(Buffer.from(await app.objectURLs.get(saved.url).arrayBuffer()),bytes);
  oneExactCommit(app);await app.run('dirty=false;navigate("Overview")');assert.equal(app.objectURLs.size,0);
});

for (const defect of ["media","hash","oversize"]) {
  test("S6 committed artifact "+defect+" rejection creates no download and never repeats commit",async()=>{
    const data=defect==="oversize"?Buffer.alloc(1048577):Buffer.from("{}");const {app,download}=await s6DownloadApp(()=>({download_blob:new Blob([data]),download_media_type:defect==="media"?"text/html":"application/json"}));
    await download.fire("click");assert.equal(app.document.downloads.length,0);assert.equal(app.objectURLs.size,0);assert.equal(app.document.activeElement===app.byId("error"),true);oneExactCommit(app);
  });
}

test("S6 obsolete download failure after navigation neither publishes bytes nor moves new focus",async()=>{
  const delayed=deferred();const {app,download}=await s6DownloadApp(()=>delayed.promise);download.focus();const action=download.fire("click");await settle();await app.run('dirty=false;navigate("Overview")');
  const focus=app.document.activeElement;const status=app.byId("status").textContent;
  delayed.resolve({status:409,download_blob:new Blob(["{}"]),download_media_type:"application/json"});await action;
  assert.equal(app.document.downloads.length,0);assert.equal(app.document.activeElement===focus,true);assert.equal(app.byId("status").textContent,status);assert.equal(app.byId("error").hidden,true);oneExactCommit(app);
});


test("S6 duplicate pending preparation remains one POST and dismissal sends no commit",async()=>{
  const delayed=deferred();const app=await s6App({"/project/bundle-imports":()=>delayed.promise});const parts=await s6Choose(app);parts.import.focus();const first=parts.import.fire("click");await settle();
  await parts.import.fire("click");assert.equal(app.requests.filter(request=>request.route==="/project/bundle-imports").length,1);assert.equal(parts.panel.getAttribute("aria-busy"),"true");assert.equal(app.document.activeElement===parts.import,true);
  delayed.resolve(s6Replacement());await first;await app.byButton("Keep editing",app.byId("preview-dialog")).fire("click");assert.equal(app.document.activeElement===parts.import,true);assert.equal(app.requests.some(request=>request.route==="/effects/commits"),false);
});

test("S6 in-progress reservation retry retains raw bytes/key while typed rejection offers a fresh request",async()=>{
  let attempts=0;const app=await s6App({"/project/bundle-imports":()=>++attempts===1?{status:409,body:{code:"bundle-preparation-in-progress",message:"Synthetic reserved preparation",retryable:true}}:{status:422,body:{code:"validation-failed",message:"Synthetic admission rejection",retryable:false}}});
  const parts=await s6Choose(app);await parts.import.fire("click");const recovery=requestParts(app);assert.match(recovery.status.textContent,/remains in progress/);
  await app.byButton("Retry the same request",recovery.row).fire("click");const requests=app.requests.filter(request=>request.route==="/project/bundle-imports");assert.equal(requests.length,2);assert.equal(requests[0].options.body===requests[1].options.body,true);assert.equal(requests[0].options.headers["Idempotency-Key"],requests[1].options.headers["Idempotency-Key"]);
  assert.match(recovery.status.textContent,/server rejected/);assert.equal(app.byId("preview-dialog").open,false);assert.equal(app.requests.some(request=>request.route==="/effects/commits"),false);
});


// Chosen-file failures remain in the installed form and never dispatch a preparation.
for (const defect of ["unreadable-file", "oversized-file"]) {
  test("S6 current chosen-file failure focuses its local alert without dispatch: "+defect,async()=>{
    const app=await s6App();
    const parts=await s6Choose(app,Buffer.from("{}"),1,async()=>{throw new Error("Synthetic local file read failure");});
    if(defect==="oversized-file")parts.file.files[0].size=1048497;
    parts.import.focus();await parts.import.fire("click");
    assert.equal(parts.error.hidden,false);
    assert.equal(app.document.activeElement===parts.error,true);
    assert.equal(parts.panel.getAttribute("aria-busy"),"false");
    assert.equal(app.requests.some(request=>request.route==="/project/bundle-imports"),false);
    assert.equal(app.byId("preview-dialog").open,false);
    assert.equal(app.byId("error").hidden,true);
  });
}


// Source API2.3 controls execute the complete selected asset with synthetic transport only.
// They are authored proposals until Root runs Node; none proves native transaction authority.

/** Hash ordered normalized index fields with the published LF encoding for synthetic DTO facts. */
function source23IndexHash(index) {
  const ordered={schema_version:index.schema_version,label:index.label,resources:index.resources.map(row=>({key:row.key,role:row.role,path:row.path}))};
  return createHash("sha256").update(JSON.stringify(ordered,null,2)+"\n").digest("hex");
}

/** Construct a complete four-path union with overwrite, new directory, removed membership and index last. */
function source23Reply() {
  const previous={schema_version:"forge.workspace/1",label:"Previous synthetic source index",resources:[
    {key:"keep",role:"policy-source",path:"policy.md"},{key:"remove",role:"policy-source",path:"retired.md"}]};
  const proposed={schema_version:"forge.workspace/2",label:"Proposed <script> literal source index",resources:[
    {key:"keep",role:"policy-source",path:"policy.md"},{key:"new",role:"policy-source",path:"new/policy.md"}]};
  const hash=source23IndexHash(proposed);const validation={state:"valid",error_count:0,warning_count:0,diagnostics:[]};
  /** Retain a current registered or explicitly unregistered generation without inventing proposed identity. */
  const current=(key,sha,size,id)=>({key,role:key===null?null:"policy-source",sha256:sha,size,resource_id:id});
  /** Describe exact proposed bytes independently of a current native generation. */
  const future=(key,sha,size)=>({key,role:key===null?null:"policy-source",sha256:sha,size});
  /** Keep every target closed and bind its current base and safe literal diff. */
  const target=(path,key,sha,size,base,baseSize)=>({path,kind:key===null?"index":"resource",key,role:key===null?null:"policy-source",
    status:base===null?"create":"overwrite",base_sha256:base,base_size:baseSize,target_version:"e".repeat(64),exact_bytes_sha256:sha,size,
    diff_text:"Synthetic <script> literal diff; no authentic source bytes.",diff_truncated:false,binary:false});
  const preview={preview_id:"prev_abcdef123456",operation_id:"op_123456abcdef",operation_type:"project-source-restore",
    snapshot_version:"a".repeat(64),observed_batch_version:"b".repeat(64),exact_manifest_sha256:"c".repeat(64),
    targets:[target("policy.md","keep","d".repeat(64),12,"1".repeat(64),10),target("new/policy.md","new","f".repeat(64),14,null,null),target("forge.workspace.json",null,hash,400,"2".repeat(64),300)],
    input_bindings:[{path:"policy.md",kind:"resource",current:current("keep","1".repeat(64),10,"res_aaaaaaaaaaaa"),proposed:future("keep","d".repeat(64),12)},
      {path:"retired.md",kind:"resource",current:current("remove","3".repeat(64),16,"res_bbbbbbbbbbbb"),proposed:null},
      {path:"new/policy.md",kind:"resource",current:null,proposed:future("new","f".repeat(64),14)},
      {path:"forge.workspace.json",kind:"index",current:current(null,"2".repeat(64),300,null),proposed:future(null,hash,400)}],
    directories:[{path:"new",status:"create",nearest_existing_parent_version:"4".repeat(64)}],validation,
    semantic_summary:"Synthetic complete source restore; no domain approval.",receipt:{token:"synthetic-private-source-receipt-token",expires_at:"2026-10-03T01:00:00Z"}};
  return {validation,preview,replacement:{previous_index:previous,proposed_index:proposed,supplied_index_sha256:hash,proposed_index_sha256:hash,removed_resource_keys:["remove"],consumed_file_count:4}};
}

/** Produce a closed specialized restore outcome with exact ordered committed targets only on commitment. */
function source23Operation(state="pending", reply=source23Reply(), extra={}) {
  const committed=state==="succeeded";const rollback=["failed","cancelled"].includes(state);
  return {operation_id:reply.preview.operation_id,kind:"bundle-restore",state,created_at:"2026-10-03T00:00:00Z",updated_at:"2026-10-03T00:00:01Z",cancel_requested:false,
    write_outcome:committed?"committed":rollback?"none":state==="recovery-required"?"unknown":"unmeasured",progress:null,
    result:committed?{write_committed:true,exact_manifest_sha256:reply.preview.exact_manifest_sha256,committed_targets:reply.preview.targets.map(row=>({path:row.path,sha256:row.exact_bytes_sha256,size:row.size})),cleanup_state:"verified"}:null,
    error:state==="failed"||state==="recovery-required"?{code:"recovery-required",message:"Synthetic safe recovery outcome.",retryable:false}:null,
    cleanup_state:committed||rollback?"verified":state==="recovery-required"?"unverified":"unmeasured",...extra};
}

/** Unlock the real selected-major handlers and install both old metadata and new source panels. */
async function source23App(overrides={},readOnly=false,version="2.3.0") {
  const reply=source23Reply();
  return s6App({"/project/source-bundle-imports":()=>reply,"/project/source-bundle-exports":()=>operation("cancelled",{kind:"export"}),
    ["/project/bundle-restores/"+reply.preview.operation_id]:()=>source23Operation("pending",reply),...overrides},readOnly,version);
}

/** Resolve only the source panel's actual labeled fields and connected controls. */
function source23Parts(app) {
  const panel=app.byId("view").querySelector("[data-source-bundle-effects]");assert(panel,"Missing source API2.3 panel");
  return {panel,target:app.byLabel("Source export project-relative target"),exportAck:app.byLabel("Include exact source bytes and sensitive metadata in this export"),
    file:app.byLabel("Choose an exact source bundle JSON file"),schema:app.byLabel("Source restore target index schema"),
    indexAck:app.byLabel("I acknowledge replacing the complete project index label and registrations"),sourceAck:app.byLabel("I acknowledge including exact source bytes and sensitive metadata"),
    filesAck:app.byLabel("I understand the complete preview may create or overwrite project files"),lookup:app.byLabel("Source restore outcome operation ID"),
    prepare:app.byButton("Prepare source restore",panel),export:app.byButton("Prepare source export",panel),lookupButton:app.byButton("Look up source restore outcome",panel),error:panel.querySelector('[role="alert"]')};
}

/** Choose exact opaque bytes, then make all three acknowledgments after actual selection invalidation. */
async function source23Choose(app,bytes=Buffer.from("{}"),version=2,read) {
  const parts=source23Parts(app);const buffer=Buffer.from(bytes);
  parts.file.files=[{name:"synthetic-source.json",size:buffer.length,
    /** Return precisely the authored File bytes, or hold this read to test ownership retirement. */
    async arrayBuffer(){return read?read():buffer.buffer.slice(buffer.byteOffset,buffer.byteOffset+buffer.byteLength);}}];
  await parts.file.fire("change");parts.schema.value=String(version);await parts.schema.fire("change");
  for(const ack of [parts.indexAck,parts.sourceAck,parts.filesAck]){ack.checked=true;await ack.fire("change");}
  return parts;
}

/** Open the actual complete source dialog through one real preparation request. */
async function source23Open(app,bytes=Buffer.from("{}")) {
  const parts=await source23Choose(app,bytes);parts.prepare.focus();await parts.prepare.fire("click");
  assert.equal(app.byId("preview-dialog").open,true);return parts;
}

/** Resolve the persistent nonauthorizing row by its preallocated ID, never by a private receipt. */
function source23RowParts(app,id=source23Reply().preview.operation_id) {
  const row=app.document.querySelector('[data-source-restore-row="'+id+'"]');assert(row,"Missing retained source outcome row");
  return {row,status:row.querySelector("[data-source-restore-status]"),error:row.querySelector("[data-source-restore-error]"),
    check:app.byButton("Check source restore status",row),cancel:app.byButton("Request source restore cancellation",row)};
}

/** Deliver synthetic facts to the actual full-asset checker/row transition inside its owning realm. */
function source23Accept(app,value,reply=source23Reply()) {
  app.run("globalThis.source23Value="+JSON.stringify(value)+";globalThis.source23Expected="+JSON.stringify(reply.preview)+";");
  return app.run("acceptSourceRestore(sourceRestoreRow(source23Value.operation_id,source23Expected),source23Value)");
}

/** Acknowledge the complete displayed plan using the real dialog's final field. */
async function source23Acknowledge(app) {
  const ack=app.byLabel("I reviewed every target, input binding, directory and index replacement; restore these exact bytes");ack.checked=true;await ack.fire("change");
  return app.byButton("Confirm this exact source restore",app.byId("preview-dialog"));
}

/** Restore one synthetic prepared context through the actual dialog function, without sending a new preparation. */
async function source23Reopen(app,reply=source23Reply()) {
  app.run("globalThis.source23Context="+JSON.stringify({family:"source-restore",preview:reply.preview,replacement:reply.replacement})+";");
  return app.run("previewSourceRestore(source23Context,()=>!stopped)");
}

/** Drive the actual pending-request Review gate, including explicit discard when the real form is dirty. */
async function source23ReviewReady(app) {
  const pendingRow=requestParts(app);const action=app.byButton("Review prepared write",pendingRow.row).fire("click");await settle();
  const openDialog=app.document.querySelector('dialog[open]');
  const discard=openDialog?.getAttribute("aria-labelledby")==="discard-title"?openDialog:null;
  if(discard)await app.byButton("Discard edits",discard).fire("click");
  await action;assert.equal(app.byId("preview-dialog").open,true);return pendingRow;
}

for(const version of ["2.0.0","2.1.0","2.2.0","2.3.0"]) {
  test("Source2.3 panel requires exact contract while old metadata remains selected: "+version,async()=>{
    const app=await source23App({},false,version);
    assert.equal(!!app.byId("view").querySelector("[data-source-bundle-effects]"),version==="2.3.0");
    assert.equal(!!app.byId("view").querySelector("[data-bundle-effects]"),["2.2.0","2.3.0"].includes(version));
    assert.equal(app.requests.some(row=>row.route.startsWith("/project/source-bundle-")),false);
  });
}

test("Source2.3 readonly preparation and cancellation abstain while explicit lookup remains available",async()=>{
  const app=await source23App({},true);const parts=await source23Choose(app);parts.target.value="exports/source.json";parts.exportAck.checked=true;await parts.exportAck.fire("change");
  await parts.prepare.fire("click");await parts.export.fire("click");assert.equal(app.requests.some(row=>row.route.startsWith("/project/source-bundle-")),false);
  source23Accept(app,source23Operation());const row=source23RowParts(app);assert.equal(row.cancel.getAttribute("aria-disabled"),"true");await row.cancel.fire("click");await row.check.fire("click");
  assert.equal(app.requests.filter(item=>item.route.includes("bundle-restores")&&item.options.method==="GET").length,1);assert.equal(app.requests.some(item=>item.route.endsWith("/cancel")),false);
});

test("Source2.3 all three acknowledgments are independent and file/schema changes clear them before dispatch",async()=>{
  const app=await source23App();const parts=await source23Choose(app);
  for(const ack of [parts.indexAck,parts.sourceAck,parts.filesAck]){ack.checked=false;await ack.fire("change");await parts.prepare.fire("click");assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);ack.checked=true;await ack.fire("change");}
  parts.schema.value="1";await parts.schema.fire("change");assert([parts.indexAck,parts.sourceAck,parts.filesAck].every(ack=>!ack.checked));
  await parts.prepare.fire("click");assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);
});

test("Source2.3 raw147 envelope retains BOM Unicode and duplicate keys without parsing chosen bytes",async()=>{
  const raw=Buffer.from('\ufeff{"key":"π","key":"unchanged duplicate"}');const app=await source23App();const parts=await source23Choose(app,raw);await parts.prepare.fire("click");
  const sent=app.requests.find(row=>row.route==="/project/source-bundle-imports");assert(sent);const actual=Buffer.from(await sent.options.body.arrayBuffer());
  const prefix=Buffer.from('{"bundle":');const suffix=Buffer.from(',"target_index_schema_version":2,"acknowledge_index_replacement":true,"acknowledge_source_content":true,"acknowledge_replace_files":true}');
  assert.equal(prefix.length+suffix.length,147);assert.deepEqual(actual,Buffer.concat([prefix,raw,suffix]));assert.equal(actual.length,raw.length+147);assert.match(sent.options.headers["Idempotency-Key"],/^[a-f0-9-]{36}$/);
});

for(const excess of [0,1]) {
  test("Source2.3 raw body cap "+(excess?"rejects excess before read":"permits exact1MiB"),async()=>{
    const app=await source23App();app.run(`globalThis.source23Reads=0;globalThis.source23File={size:${1048429+excess},arrayBuffer:async()=>{source23Reads++;return new Uint8Array(${1048429+excess}).buffer;}}`);
    if(excess){await assert.rejects(app.run("sourceBundleImportBody(source23File,2)"),/1048429/);assert.equal(app.run("source23Reads"),0);}
    else{const body=await app.run("sourceBundleImportBody(source23File,2)");assert.equal(body.size,1048576);assert.equal(app.run("source23Reads"),1);}
    assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);
  });
}

test("Source2.3 late File bytes lose ownership after reselection and never dispatch",async()=>{
  const read=deferred();const app=await source23App();const parts=await source23Choose(app,Buffer.from("{}"),2,()=>read.promise);parts.prepare.focus();const action=parts.prepare.fire("click");await settle();
  await source23Choose(app,Buffer.from('{"new":true}'));read.resolve(new Uint8Array([123,125]).buffer);await action;
  assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);assert.equal(parts.panel.getAttribute("aria-busy"),"false");assert.equal(parts.prepare.getAttribute("aria-disabled"),"false");
});

test("Source2.3 complete preview renders all bases targets directories and index-last facts without token or HTML injection",async()=>{
  const app=await source23App();await source23Open(app);const content=app.byId("preview-content");const row=source23RowParts(app);
  assert.match(content.textContent,/retired.md/);assert.match(content.textContent,/new\/policy.md/);assert.match(content.textContent,/Nearest existing parent version/);assert.match(content.textContent,/Files removed from membership are retained on disk/);
  assert.match(content.textContent,/Every target in publication order; index last/);assert.match(content.textContent,/Proposed <script> literal source index/);assert.equal(content.querySelectorAll("script").length,0);assert.doesNotMatch(content.textContent,/synthetic-private-source-receipt-token/);
  assert.equal(row.row.getAttribute("aria-labelledby"),"source-restore-title-op_123456abcdef");assert.equal(row.status.getAttribute("aria-label"),"Source restore op_123456abcdef status");
  await app.byButton("Keep editing",app.byId("preview-dialog")).fire("click");assert.equal(app.requests.some(item=>item.route.endsWith("/commit")),false);assert(row.row.isConnected);
});

for(const defect of ["count-low","count-high","binding-dropped","wrong-hash","index-not-last","removed-key-dropped","registered-base-lost","alias-target","diff-union-excess"]) {
  test("Source2.3 malformed complete preview refuses confirmation: "+defect,async()=>{
    const reply=source23Reply();if(defect==="count-low")reply.replacement.consumed_file_count=3;if(defect==="count-high")reply.replacement.consumed_file_count=5;
    if(defect==="binding-dropped")reply.preview.input_bindings.splice(1,1);if(defect==="wrong-hash")reply.replacement.proposed_index_sha256="9".repeat(64);
    if(defect==="index-not-last")reply.preview.targets.reverse();if(defect==="removed-key-dropped")reply.replacement.removed_resource_keys=[];
    if(defect==="registered-base-lost")Object.assign(reply.preview.input_bindings[0].current,{key:null,role:null,resource_id:null});
    if(defect==="alias-target")reply.preview.targets[1].path="POLICY.md";
    if(defect==="diff-union-excess"){reply.preview.targets[0].diff_text="x".repeat(100001);reply.preview.targets[1].diff_text="y".repeat(100001);}
    const app=await source23App({"/project/source-bundle-imports":()=>reply});const parts=await source23Choose(app);await parts.prepare.fire("click");
    assert.equal(app.byId("preview-dialog").open,false);assert.equal(app.requests.some(item=>item.route.endsWith("/commit")||item.route==="/effects/commits"),false);assert(requestParts(app).row.isConnected);
  });
}

test("Source2.3 one exact confirmation forwards token batch and acknowledgment then observes only its known ID",async()=>{
  const reply=source23Reply();const route="/project/bundle-restores/"+reply.preview.operation_id;let reads=0;
  const app=await source23App({["/project/bundle-restores/"+reply.preview.preview_id+"/commit"]:()=>source23Operation("pending",reply),[route]:()=>{reads++;return source23Operation("succeeded",reply);}});
  await source23Open(app);const confirm=await source23Acknowledge(app);confirm.focus();await confirm.fire("click");await settle();
  const sent=app.requests.filter(item=>item.route.endsWith("/commit"));assert.equal(sent.length,1);assert.deepEqual(JSON.parse(sent[0].options.body),{receipt:reply.preview.receipt.token,observed_batch_version:reply.preview.observed_batch_version,acknowledge_exact_restore:true});
  assert.match(sent[0].options.headers["Idempotency-Key"],/^[a-f0-9-]{36}$/);assert.equal(reads,1);assert(app.requests.filter(item=>item.route===route).every(item=>item.options.method==="GET"));
  const row=source23RowParts(app);assert.match(row.status.textContent,/Exact source bytes committed.*Cleanup verified/);assert.equal(app.byId("preview-dialog").open,false);assert.equal(app.requests.some(item=>item.route==="/effects/commits"),false);
});

test("Source2.3 pending Review and lost confirmation retain one attempt across dismissal and reopening",async()=>{
  const reply=source23Reply();const preparation=deferred();const app=await source23App({"/project/source-bundle-imports":()=>preparation.promise,
    ["/project/bundle-restores/"+reply.preview.preview_id+"/commit"]:()=>({status:503,body:{code:"recovery-required",message:"Synthetic lost acceptance reply.",retryable:false}})});
  const parts=await source23Choose(app);const action=parts.prepare.fire("click");await settle();parts.schema.value="1";await parts.schema.fire("change");preparation.resolve(reply);await action;
  assert.equal(app.byId("preview-dialog").open,false);await source23ReviewReady(app);const confirm=await source23Acknowledge(app);await confirm.fire("click");
  const row=source23RowParts(app);assert.match(row.status.textContent,/outcome is unverified/);assert.equal(row.row.isConnected,true);await app.byButton("Keep editing",app.byId("preview-dialog")).fire("click");
  await source23Reopen(app,reply);const second=await source23Acknowledge(app);assert.equal(second.getAttribute("aria-disabled"),"true");await second.fire("click");
  assert.equal(app.requests.filter(item=>item.route.endsWith("/commit")).length,1);assert.equal(app.requests.filter(item=>item.route==="/project/source-bundle-imports").length,1);
});

for(const seam of ["GET","cancel","commit"]) {
  test("Source2.3 late "+seam+" failure cannot erase separately verified committed outcome",async()=>{
    const reply=source23Reply();const delayed=deferred();const route="/project/bundle-restores/"+reply.preview.operation_id;
    const app=await source23App({[route]:()=>delayed.promise,[route+"/cancel"]:()=>delayed.promise,["/project/bundle-restores/"+reply.preview.preview_id+"/commit"]:()=>delayed.promise});
    let action;if(seam==="commit"){await source23Open(app);const confirm=await source23Acknowledge(app);action=confirm.fire("click");}
    else{source23Accept(app,source23Operation("pending",reply));const row=source23RowParts(app);action=(seam==="GET"?row.check:row.cancel).fire("click");}
    await settle();source23Accept(app,source23Operation("succeeded",reply));const before=source23RowParts(app).status.textContent;
    delayed.resolve({status:503,body:{code:"recovery-required",message:"Synthetic stale transport failure.",retryable:false}});await action;
    assert.equal(source23RowParts(app).status.textContent,before);assert.equal(app.run('sourceRestoreRows.get("op_123456abcdef").last.state'),"succeeded");assert.equal(app.run('sourceRestoreRows.get("op_123456abcdef").last.write_outcome'),"committed");
  });
}

test("Source2.3 active counters describe staging and reject regressions or denominator changes",async()=>{
  const app=await source23App();const pending=source23Operation("running",source23Reply(),{progress:{completed_files:1,total_files:3}});source23Accept(app,pending);const row=source23RowParts(app);
  assert.match(row.status.textContent,/Staged files: 1 of 3 reported/);assert.doesNotMatch(row.status.textContent,/bytes committed|published/);
  assert.throws(()=>source23Accept(app,{...pending,progress:{completed_files:0,total_files:3}}),/staging counters/);assert.throws(()=>source23Accept(app,{...pending,progress:{completed_files:1,total_files:4}}),/staging counters/);
  assert.match(row.status.textContent,/Staged files: 1 of 3 reported/);
});

test("Source2.3 committed cleanup degradation remains committed and accepts explicit recovery-required",async()=>{
  const app=await source23App();const succeeded=source23Operation("succeeded");succeeded.cleanup_state="pending";succeeded.result.cleanup_state="pending";source23Accept(app,succeeded);
  assert.match(source23RowParts(app).status.textContent,/Exact source bytes committed.*Cleanup pending/);
  const recovery={...succeeded,state:"recovery-required",updated_at:"2026-10-03T00:00:02Z",cleanup_state:"unverified",result:{...succeeded.result,cleanup_state:"unverified"},error:{code:"recovery-required",message:"Synthetic cleanup uncertainty.",retryable:false}};
  source23Accept(app,recovery);assert.match(source23RowParts(app).status.textContent,/requires recovery.*Write outcome: committed.*Cleanup unverified/);assert.match(source23RowParts(app).row.textContent,/Complete committed source targets/);
});

test("Source2.3 verified cancellation with null error reports rollback without inventing publication",async()=>{
  const app=await source23App();source23Accept(app,source23Operation("cancelled",source23Reply(),{cancel_requested:true}));const row=source23RowParts(app);
  assert.match(row.status.textContent,/cancelled; verified rollback reports no committed write/);assert.equal(row.error.hidden,true);assert.equal(row.cancel.getAttribute("aria-disabled"),"true");
});

test("Source2.3 result replacement transfers only focus owned by the replaced result subtree",async()=>{
  const app=await source23App();const success=source23Operation("succeeded");source23Accept(app,success);app.byId("view-title").focus();const outside=app.document.activeElement;
  const pending={...success,updated_at:"2026-10-03T00:00:02Z",cleanup_state:"pending",result:{...success.result,cleanup_state:"pending"}};source23Accept(app,pending);assert.equal(app.document.activeElement,outside);
  app.run('globalThis.source23Focus=node("button","Synthetic result focus");sourceRestoreRows.get("op_123456abcdef").result.append(source23Focus);source23Focus.focus();');
  const verified={...success,updated_at:"2026-10-03T00:00:03Z"};source23Accept(app,verified);assert.equal(app.document.activeElement,source23RowParts(app).status);
});

test("Source2.3 shutdown retains public lookup ID while suppressing an in-flight check and all old receipts",async()=>{
  const late=deferred();const reply=source23Reply();const route="/project/bundle-restores/"+reply.preview.operation_id;
  const app=await source23App({[route]:()=>late.promise,"/session/shutdown":()=>({state:"shutting-down"})});source23Accept(app,source23Operation());const row=source23RowParts(app);const checking=row.check.fire("click");await settle();
  await app.byId("stop").fire("click");await app.byId("confirm-stop").fire("click");const count=app.requests.length;const status=row.status.textContent;
  late.resolve(source23Operation("succeeded"));await checking;await row.check.fire("click");await row.cancel.fire("click");
  assert.equal(row.row.isConnected,true);assert.match(status,/Workspace stopped.*fresh same-project API2 2.3.0.*not-found does not prove no write/);assert.equal(row.status.textContent,status);assert.equal(app.requests.length,count);assert.equal(app.run("capability"),"");
});

test("Source2.3 fresh readonly lookup404 retains known ID and never dispatches restore or cancellation",async()=>{
  const reply=source23Reply();const route="/project/bundle-restores/"+reply.preview.operation_id;
  const app=await source23App({[route]:()=>({status:404,body:{code:"not-found",message:"No retained synthetic outcome.",retryable:false}})},true);const parts=source23Parts(app);parts.lookup.value=reply.preview.operation_id;await parts.lookup.fire("input");parts.lookupButton.focus();await parts.lookupButton.fire("click");
  const row=source23RowParts(app);assert.match(row.status.textContent,/missing outcome is not proof.*do not resend/);assert.equal(row.row.isConnected,true);assert.equal(app.requests.filter(item=>item.route===route).length,1);
  assert.equal(app.requests.some(item=>item.route.includes("bundle-restores")&&item.options.method==="POST"||item.route.startsWith("/project/source-bundle-")),false);
});

test("Source2.3 delayed preview hash cannot open after a newer ordinary preview and Escape",async()=>{
  const app=await source23App();const old=source23Reply();const ordinary=proposedWrite();app.routes["/effects/previews/"+ordinary.preview_id]=()=>ordinary;
  app.run('globalThis.source23Subtle=crypto.subtle;globalThis.source23Release=null;crypto.subtle={digest:async(...args)=>{await new Promise(resolve=>{source23Release=resolve;});return source23Subtle.digest(...args);}};');
  const pending=source23Reopen(app,old);await settle();assert.equal(typeof app.run("source23Release"),"function");
  app.run("globalThis.source23Ordinary="+JSON.stringify(ordinary)+";");await app.run("preview(source23Ordinary)");assert.equal(app.byId("preview-dialog").open,true);app.byId("preview-dialog").escape();await settle();app.run("source23Release()");
  assert.equal(await pending,false);assert.equal(app.byId("preview-dialog").open,false);assert.equal(app.requests.some(item=>item.route.endsWith("/commit")),false);
});

test("Source2.3 older preparation ready after newer preview dismissal requires explicit Review",async()=>{
  const delayed=deferred();const reply=source23Reply();const ordinary=proposedWrite();const app=await source23App({"/project/source-bundle-imports":()=>delayed.promise,["/effects/previews/"+ordinary.preview_id]:()=>ordinary});
  const parts=await source23Choose(app);const preparing=parts.prepare.fire("click");await settle();app.run("globalThis.source23Ordinary="+JSON.stringify(ordinary)+";");await app.run("preview(source23Ordinary)");app.byId("preview-dialog").escape();await settle();
  delayed.resolve(reply);await preparing;assert.equal(app.byId("preview-dialog").open,false);assert.match(requestParts(app).status.textContent,/ready for review/);assert.equal(source23RowParts(app).row.isConnected,true);
  await source23ReviewReady(app);assert.match(app.byId("preview-content").textContent,/Review complete exact source restore/);assert.equal(app.requests.filter(item=>item.route==="/project/source-bundle-imports").length,1);
});

/** Prepare and confirm a synthetic source export through unchanged generic effects before exact-byte download. */
async function source23DownloadApp(override) {
  const index={schema_version:"forge.workspace/2",label:"Synthetic source export",resources:[]};const bytes=Buffer.from(JSON.stringify({schema_version:"forge.workspace-index-bundle/3",profile:"index-and-source-hex",source_content_included:true,index,index_sha256:source23IndexHash(index),pins:[],contents:[]}));
  const preview={...proposedWrite(),operation_type:"report-export",target:{status:"create",path:"exports/source.json"},exact_bytes_sha256:createHash("sha256").update(bytes).digest("hex")};
  const app=await source23App({"/project/source-bundle-exports":()=>operation("succeeded",{kind:"export",result:{preview}}),["/effects/previews/"+preview.preview_id]:()=>preview,
    "/effects/commits":()=>operation("pending",{kind:"commit"}),[operationRoute]:()=>s6Committed(preview),
    ["/project/source-bundle-exports/"+operationId+"/download"]:override??(()=>({download_blob:new Blob([bytes]),download_media_type:"application/json; charset=utf-8"}))});
  const parts=source23Parts(app);parts.target.value="exports/source.json";await parts.target.fire("input");parts.exportAck.checked=true;await parts.exportAck.fire("change");parts.export.focus();await parts.export.fire("click");
  assert.equal(app.byId("preview-dialog").open,true);await app.byButton("Confirm this exact write",app.byId("preview-dialog")).fire("click");return {app,bytes,download:app.byButton("Download committed source bundle")};
}

test("Source2.3 committed private JSON download binds family authentication exact hash and fixed filename",async()=>{
  const {app,bytes,download}=await source23DownloadApp();download.focus();await download.fire("click");const sent=app.requests.find(item=>item.route.endsWith("/download"));
  assert.equal(sent.route,"/project/source-bundle-exports/"+operationId+"/download");assert.equal(sent.options.credentials,"omit");assert.equal(sent.options.redirect,"error");assert.match(sent.options.headers.Authorization,/^Bearer /);
  assert.equal(app.document.downloads.length,1);const saved=app.document.downloads[0];assert.equal(saved.filename,"forge-workspace-index-and-source-content.json");assert.deepEqual(Buffer.from(await app.objectURLs.get(saved.url).arrayBuffer()),bytes);oneExactCommit(app);
  await app.run('dirty=false;navigate("Overview")');assert.equal(app.objectURLs.size,0);
});

for(const defect of ["media","hash","oversize"]) {
  test("Source2.3 committed download "+defect+" refuses publication without repeating generic commit",async()=>{
    const bytes=defect==="oversize"?Buffer.alloc(1048430):Buffer.from("{}");const {app,download}=await source23DownloadApp(()=>({download_blob:new Blob([bytes]),download_media_type:defect==="media"?"text/html":"application/json"}));
    await download.fire("click");assert.equal(app.document.downloads.length,0);assert.equal(app.objectURLs.size,0);assert.equal(app.document.activeElement,app.byId("error"));oneExactCommit(app);
  });
}


// These two controls target the recorded V2 ownership repair without replacing old assertions.
test("Source2.3 newer preview retires a deferred file read before any preparation POST",async()=>{
  const reading=deferred();const app=await source23App();const parts=await source23Choose(app,Buffer.from("{}"),2,()=>reading.promise);
  const preparing=parts.prepare.fire("click");await settle();await source23Reopen(app);assert.equal(app.byId("preview-dialog").open,true);
  app.byId("preview-dialog").escape();await settle();reading.resolve(new Uint8Array([123,125]).buffer);await preparing;
  assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);assert.equal(app.byId("preview-dialog").open,false);assert.equal(parts.panel.getAttribute("aria-busy"),"false");
});

test("Source2.3 newer preview during request pacing retires unsent recovery without fetch",async()=>{
  const app=await source23App();const parts=await source23Choose(app);app.run("nextRequestAt=performance.now()+1200;");
  const preparing=parts.prepare.fire("click");await settle();assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);assert(app.timers.length>0,"Actual API pacing must be waiting");
  await source23Reopen(app);app.byId("preview-dialog").escape();await settle();await app.poll();await preparing;
  assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);assert.equal(app.run("requestRows.size"),0);assert.equal(app.byId("preview-dialog").open,false);assert.equal(parts.panel.getAttribute("aria-busy"),"false");
});


// This final case consumes the separately frozen V3 obsolete-error ownership repair.
test("Source2.3 rejected dispatched preparation stays in recovery without altering a newer preview or focus",async()=>{
  const rejected=deferred();const ordinary=proposedWrite();const app=await source23App({"/project/source-bundle-imports":()=>rejected.promise,["/effects/previews/"+ordinary.preview_id]:()=>ordinary});
  const parts=await source23Choose(app);parts.prepare.focus();const preparing=parts.prepare.fire("click");await settle();
  assert.equal(app.requests.filter(row=>row.route==="/project/source-bundle-imports").length,1,"The old preparation must already be dispatched");const epoch=app.run("pending");const recovery=requestParts(app);
  app.run("globalThis.source23Ordinary="+JSON.stringify(ordinary)+";");await app.run("preview(source23Ordinary)");const dialog=app.byId("preview-dialog");assert.equal(dialog.open,true);assert.equal(dialog.querySelector('[role="alert"]'),null);
  const owned=app.byButton("Confirm this exact write",dialog);owned.focus();const content=app.byId("preview-content").textContent;const focusCount=app.document.focusHistory.length;
  rejected.resolve({status:503,body:{code:"query-budget-exceeded",message:"Synthetic obsolete dispatched preparation error.",retryable:true}});await preparing;
  assert.equal(app.run("pending"),epoch);assert.equal(dialog.open,true);assert.equal(dialog.querySelector('[role="alert"]'),null);assert.equal(app.byId("preview-content").textContent,content);assert.equal(app.document.activeElement,owned);assert.equal(app.document.focusHistory.length,focusCount);
  assert.equal(recovery.row.isConnected,true);assert.match(recovery.error.textContent,/Synthetic obsolete dispatched preparation error/);assert.equal(app.requests.filter(row=>row.route==="/project/source-bundle-imports").length,1);assert.equal(app.requests.some(row=>row.route.endsWith("/commit")||row.route==="/effects/commits"),false);
});


test("Source2.3 selected oversized file refuses before reading or dispatch and preserves explicit retry focus",async()=>{
  let reads=0;const app=await source23App();const parts=await source23Choose(app,Buffer.alloc(1048430),2,async()=>{reads++;return new ArrayBuffer(0);});
  parts.prepare.focus();await parts.prepare.fire("click");
  const alert=parts.panel.querySelector('[role="alert"]');assert(alert);assert.equal(alert.hidden,false);assert.match(alert.textContent,/1048429/);
  assert.equal(app.document.activeElement,alert);assert.equal(reads,0);assert.equal(parts.panel.getAttribute("aria-busy"),"false");assert.equal(parts.prepare.getAttribute("aria-disabled"),"false");
  assert.equal(app.requests.some(row=>row.route==="/project/source-bundle-imports"),false);assert.equal(app.byId("preview-dialog").open,false);assert.equal(app.run("requestRows.size"),0);
  assert.match(parts.panel.textContent,/No source preparation was confirmed.*retry explicitly/);
});
