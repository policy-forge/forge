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
const { randomUUID, createHash } = require("node:crypto");
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
  /** Assemble a connected shell with the same view, dialog, and status identifiers. */
  constructor() {
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

  /** Query the connected body tree for modal and error targets. */
  querySelector(selector) { return this.body.querySelector(selector); }
}

/** Execute the entire selected source file in an isolated realm without transformation. */
function harness(overrides = {}) {
  const document = new FakeDocument();
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
    crypto: { randomUUID },
    performance: { now: () => (clock += 100) },
    setTimeout: (callback, milliseconds = 0) => {
      if (milliseconds >= 500) timers.push(callback);
      else queueMicrotask(callback);
      return timers.length;
    },
    URLSearchParams, URL:HarnessURL, Blob, console,
    fetch: async (url, options) => {
      const parsed = new URL(url, "http://127.0.0.1:1");
      const route = parsed.pathname.replace(/^\/api\/v1/, "");
      requests.push({ route, url, options });
      assert.equal(typeof routes[route], "function", "Unexpected API route: " + route);
      const value = await routes[route](parsed, options);
      const status = value.status ?? 200;
      return { ok: status >= 200 && status < 300, json: async () => value.body ?? value };
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
