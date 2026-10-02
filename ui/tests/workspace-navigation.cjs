// Coverage follows the complete workspace.js asset beside this harness.
// In the temporary proposal layout it identifies proposed-copy execution only;
// after applying to a worktree it identifies that worktree's actual asset.
// Source-level navigation regressions. This fake DOM is not browser or AT evidence.
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { randomUUID } = require("node:crypto");

const productionPath = path.resolve(__dirname, "../workspace.js");
const productionSource = fs.readFileSync(productionPath, "utf8");

/** Construct a cancellable DOM event for the narrow interaction harness. */
function eventFor(type, target) {
  return {
    type, target, currentTarget: target, defaultPrevented: false,
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
    this.hidden = false;
    this.open = false;
    this.required = false;
    this.tabIndex = undefined;
    this._text = "";
    this._disabled = false;
    this._inert = false;
  }

  /** A detached node cannot receive focus or serve as a dialog return target. */
  get isConnected() {
    let current = this;
    while (current.parentNode) current = current.parentNode;
    return current === this.ownerDocument.body;
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

  /** Setting text replaces descendants, including any focused child. */
  set textContent(value) {
    this.replaceChildren();
    this._text = String(value);
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

  /** Close a modal, attempt native restoration, and fire production close handlers. */
  close() {
    if (!this.open) return;
    this.open = false;
    this.ownerDocument.activeElement = this.ownerDocument.body;
    this.returnTarget?.focus();
    this.dispatchEvent(eventFor("close", this));
  }

  /** Escape emits cancel and uses native non-destructive dismissal unless prevented. */
  escape() {
    const event = eventFor("cancel", this);
    if (this.dispatchEvent(event)) this.close();
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

/** Create one isolated realm and execute the entire unchanged production asset. */
function harness(overrides = {}) {
  const document = new FakeDocument();
  const requests = [];
  const routes = {
    "/project/summary": () => ({
      project_label: "Synthetic project", workspace_index_present: true,
      resource_counts: { total: 1, invalid: 0, stale: 0 },
      review_counts: { total_open: 0 }, next_action: "review-scope",
    }),
    "/project/config-status": () => ({ present: true, valid: true }),
    "/resources": () => ({ page: { items: [], total_matching: 0, next_cursor: null } }),
    "/provenance/entries": () => ({ page: { items: [], total_matching: 0, next_cursor: null } }),
    ...overrides,
  };
  let clock = 0;
  const context = vm.createContext({
    document,
    window: { addEventListener() {} },
    crypto: { randomUUID },
    performance: { now: () => (clock += 100) },
    setTimeout: callback => { queueMicrotask(callback); return 0; },
    URLSearchParams, URL, console,
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
  const run = source => vm.runInContext(source, context);
  const byId = id => document.getElementById(id);
  const byButton = (label, root = document.body) => {
    const control = root.querySelectorAll("button").find(node => node.textContent === label);
    assert(control, "Missing button: " + label);
    return control;
  };
  const byLabel = label => {
    const caption = document.body.querySelectorAll("label").find(node => node.textContent === label);
    assert(caption, "Missing label: " + label);
    return byId(caption.htmlFor);
  };
  return { document, requests, routes, run, byId, byButton, byLabel };
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
  app.run('readOnly = false; element("view").replaceChildren(evidenceList([{label:"Fixture policy",provenance_ref:"prov_fixture"}]),resourceActions([]));');
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
    items: [{ resource_id: "res_fixture", key: "fixture-policy", role: "policy-source" }],
    total_matching: 1, next_cursor: null,
  } }) });
  await app.run('navigate("Trace & Reports")');
  await app.byButton("Trace fixture-policy").fire("click");
  assert.equal(app.byId("view-title").textContent, "Provenance");
  assert.equal(app.document.activeElement, app.byId("view-title"));
  assert.match(app.requests.at(-1).url, /anchor=res_fixture/);
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
  const stale = app.run('showProvenance("prov_fixture")');
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
  const stale = app.run('showProvenance("prov_fixture")');
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
      capability: "synthetic-test-capability",
      session: { api_major: 1, read_only: true },
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
      items: [{ resource_id: "res_fixture", key: "fixture-policy", role: "policy-source" }],
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
