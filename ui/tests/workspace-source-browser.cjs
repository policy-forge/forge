/** Drive real installed Chrome source export, exact restore, and fresh read-only outcome lookup. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const [origin, stage, project, donor, shared, out, root, chrome] = process.argv.slice(2);
assert.match(origin, /^http:\/\/127\.0\.0\.1:[0-9]{1,5}$/);
assert(['export', 'restore', 'lookup'].includes(stage));
assert([project, donor, shared, out, root, chrome].every(value => path.isAbsolute(value)));
assert(!fs.existsSync(out));
fs.mkdirSync(out, { mode: 0o700 });
const passphrase = process.env.FORGE_SOURCE_BROWSER_PASSPHRASE;
delete process.env.FORGE_SOURCE_BROWSER_PASSPHRASE;
assert(typeof passphrase === 'string' && /^synthetic-source-bundle-[0-9a-f]{48}$/.test(passphrase));
assert.equal(process.version, 'v24.19.0');
const { chromium } = require(path.join(root, 'ui/node_modules/playwright'));

/** Hash exact bytes without persisting source contents or credential-bearing objects. */
function sha(bytes) { return createHash('sha256').update(bytes).digest('hex'); }

/** Inventory only the owned small project, rejecting aliases and retaining bounded exact byte facts. */
function inventory(directory) {
  const result = {};
  let entries = 0;
  let total = 0;
  /** Walk owned directories with explicit depth, entry, file and aggregate limits. */
  function visit(base, prefix, depth) {
    assert(depth <= 8);
    const names = fs.readdirSync(base).sort();
    assert(names.length <= 32);
    for (const name of names) {
      assert(++entries <= 64);
      const full = path.join(base, name);
      const key = prefix + name;
      const stat = fs.lstatSync(full);
      assert(!stat.isSymbolicLink());
      if (stat.isDirectory()) visit(full, key + '/', depth + 1);
      else {
        assert(stat.isFile() && stat.size <= 2 * 1024 * 1024);
        total += stat.size;
        assert(total <= 8 * 1024 * 1024);
        const raw = fs.readFileSync(full);
        assert.equal(raw.length, stat.size);
        result[key] = { sha256: sha(raw), size: raw.length };
      }
    }
  }
  visit(directory, '', 0);
  return result;
}

/** Retain only a closed safe outcome projection, never an acceptance receipt, source body or error text. */
function safeOutcome(value) {
  assert(value && /^op_[0-9a-z]{12,80}$/.test(value.operation_id));
  return { operation_id: value.operation_id, kind: value.kind, state: value.state,
    write_outcome: value.write_outcome, cleanup_state: value.cleanup_state,
    cancel_requested: value.cancel_requested, error_code: value.error?.code ?? null,
    result: value.result === null ? null : {
      exact_manifest_sha256: value.result.exact_manifest_sha256,
      committed_targets: value.result.committed_targets,
      cleanup_state: value.result.cleanup_state,
    } };
}

/** Create evidence exactly once in the private output directory. */
function writeJSON(filename, value) {
  fs.writeFileSync(filename, JSON.stringify(value, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
}

const sourceJS = fs.readFileSync(path.join(root, 'ui/workspace.js'));
const sourceCSS = fs.readFileSync(path.join(root, 'ui/workspace.css'));
const before = inventory(project);
const donorBefore = inventory(donor);
const receipt = { format: 'forge.s6-source-native-browser/1', status: 'failed', stage,
  api_major: 2, api_version: null, phase: 'launch',
  driver_sha256: sha(fs.readFileSync(__filename)), source_js_sha256: sha(sourceJS),
  source_css_sha256: sha(sourceCSS), node_version: process.version, fixture_before: before,
  focus_observations: [], reflow: [], screenshots: [], requests: [], restore_commit_requests: 0,
  page_errors: 0, non_loopback_page_requests: 0, last_outcome: null,
  browser_close_verified: false, context_close_verified: false,
  limitations: ['Real UI/API observations require this driver to execute; preparation supplies no results.',
    'No HTTP response or application-state injection; the fixtures are synthetic owned local files.',
    'Page route restriction is not OS network denial; viewport captures are not accessibility or human acceptance.',
    'Normal browser/context closure is not a measurement of an empty OS descendant tree.',
    'The native per-user journal state and its retirement are outside the fixture inventory.'] };

(async () => {
  let browser;
  let context;
  let page;
  let known;
  let observedOutcome;
  let observerError = false;
  const pendingObservations = new Set();
  try {
    browser = await chromium.launch({ headless: true, executablePath: chrome,
      args: ['--disable-background-networking'] });
    receipt.browser_version = browser.version();
    receipt.playwright_version = JSON.parse(fs.readFileSync(path.join(root, 'ui/node_modules/playwright/package.json'))).version;
    receipt.playwright_core_version = JSON.parse(fs.readFileSync(path.join(root, 'ui/node_modules/playwright-core/package.json'))).version;
    assert.equal(receipt.playwright_version, '1.62.1');
    assert.equal(receipt.playwright_core_version, '1.62.1');
    context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, serviceWorkers: 'block', acceptDownloads: true });
    page = await context.newPage();
    page.setDefaultTimeout(20000);
    page.on('pageerror', () => receipt.page_errors++);
    const requestKinds = new Set();
    await context.route('**/*', async route => {
      const request = route.request();
      const target = new URL(request.url());
      if (target.origin !== origin) { receipt.non_loopback_page_requests++; await route.abort(); return; }
      if (target.pathname.startsWith('/api/')) {
        assert(target.pathname.startsWith('/api/v2/'));
        requestKinds.add(request.method() + ' ' + target.pathname.replace(/(?:res|op|prev)_[0-9a-z]+/g, '{id}'));
        receipt.requests = [...requestKinds].sort();
        if (request.method() === 'POST' && /^\/api\/v2\/project\/bundle-restores\/prev_[0-9a-z]+\/commit$/.test(target.pathname)) {
          receipt.restore_commit_requests++;
        }
      }
      await route.continue();
    });
    page.on('response', response => {
      const pathname = new URL(response.url()).pathname;
      if (response.request().method() !== 'GET' || !/^\/api\/v2\/project\/bundle-restores\/op_[0-9a-z]+$/.test(pathname)) return;
      const observation = (async () => {
        if (response.status() !== 200) return;
        const raw = await response.body();
        assert(raw.length <= 1024 * 1024);
        const value = JSON.parse(raw);
        receipt.last_outcome = safeOutcome(value);
        if (known && value.operation_id === known.operation_id) observedOutcome = value;
      })().catch(() => { observerError = true; });
      pendingObservations.add(observation);
      observation.finally(() => pendingObservations.delete(observation));
    });

    /** Observe connected native focus without setting the expected focus to satisfy its predicate. */
    async function focused(locator, label) {
      const end = Date.now() + 20000;
      while (!await locator.evaluate(element => element.isConnected && element === document.activeElement)) {
        assert(Date.now() < end);
        await page.waitForTimeout(25);
      }
      receipt.focus_observations.push(label);
    }

    /** Capture actual desktop and320px viewport pixels, keeping any requested outcome text in view. */
    async function captures(label, anchor = null) {
      for (const width of [1440, 320]) {
        await page.setViewportSize({ width, height: 1000 });
        await page.waitForTimeout(50);
        if (anchor) {
          await anchor.scrollIntoViewIfNeeded();
          const bounds = await anchor.evaluate(element => {
            const box = element.getBoundingClientRect();
            return { top: box.top, bottom: box.bottom, viewport: innerHeight };
          });
          assert(bounds.top >= 0 && bounds.bottom <= bounds.viewport, 'Outcome text must be inside the captured viewport');
        }
        const dimensions = await page.evaluate(() => ({ viewport: innerWidth, page: document.documentElement.scrollWidth }));
        receipt.reflow.push({ phase: label, ...dimensions });
        const name = label + '-' + width + '.png';
        await page.screenshot({ path: path.join(out, name), fullPage: false, timeout: 20000 });
        receipt.screenshots.push(name);
        assert(dimensions.page <= dimensions.viewport, 'Global page overflow');
      }
      await page.setViewportSize({ width: 1440, height: 1000 });
    }

    /** Prepare with the actual initiating control and verify the native review heading focus. */
    async function prepared(button) {
      await button.focus();
      await button.press('Enter');
      await page.locator('#preview-dialog[open]').waitFor();
      await focused(page.locator('#preview-title'), 'exact-review-heading');
    }

    /** Dismiss an unconfirmed actual preview and verify no fixture bytes changed. */
    async function dismiss(button) {
      await page.getByRole('button', { name: 'Keep editing', exact: true }).press('Enter');
      await page.locator('#preview-dialog[open]').waitFor({ state: 'hidden' });
      await focused(button, 'dismiss-return-control');
      assert.deepEqual(inventory(project), before);
    }

    /** Require the real terminal outcome and its full confirmed hash membership, with no forged success state. */
    async function succeeded(id, expected) {
      const row = page.locator('[data-source-restore-row="' + id + '"]');
      await row.locator('[data-source-restore-status]').filter({ hasText: 'Exact source bytes committed. Cleanup verified.' }).waitFor({ timeout: 90000 });
      await Promise.all([...pendingObservations]);
      assert(!observerError);
      assert(observedOutcome && observedOutcome.operation_id === id);
      assert.equal(observedOutcome.state, 'succeeded');
      assert.equal(observedOutcome.write_outcome, 'committed');
      assert.equal(observedOutcome.cleanup_state, 'verified');
      assert.equal(observedOutcome.error, null);
      assert.equal(observedOutcome.result.exact_manifest_sha256, expected.exact_manifest_sha256);
      assert.deepEqual(observedOutcome.result.committed_targets, expected.targets);
      return row;
    }

    const asset = page.waitForResponse(response => /\/assets\/[a-f0-9]{64}\.js$/.test(new URL(response.url()).pathname));
    const style = page.waitForResponse(response => /\/assets\/[a-f0-9]{64}\.css$/.test(new URL(response.url()).pathname));
    await page.goto(origin);
    assert.deepEqual(await (await asset).body(), sourceJS);
    assert.deepEqual(await (await style).body(), sourceCSS);
    assert.equal(await page.locator('meta[name="forge-api-major"]').getAttribute('content'), '2');
    const negotiated = await page.locator('meta[name="forge-api-contract-version"]').getAttribute('content');
    assert(['2.3.0','2.4.0'].includes(negotiated));receipt.api_version=negotiated;
    const unlock = page.getByLabel('Workspace passphrase');
    await focused(unlock, 'startup-passphrase');
    await unlock.fill(passphrase);
    await unlock.press('Enter');
    await page.waitForFunction(() => document.getElementById('status').textContent === 'Project state loaded.');
    const navigation = page.getByRole('button', { name: 'Trace & Reports', exact: true });
    await navigation.focus();
    await navigation.press('Enter');
    await page.waitForFunction(() => document.getElementById('view-title').textContent === 'Trace & Reports');
    await focused(page.locator('#view-title'), 'trace-heading');
    const panel = page.locator('[data-source-bundle-effects]');
    await panel.waitFor();
    assert.equal(await panel.getByRole('heading', { name: 'Export or restore exact sources', exact: true }).count(), 1);
    assert.deepEqual(inventory(project), before);

    if (stage === 'export') {
      receipt.phase = 'export-preview';
      await panel.getByLabel('Source export project-relative target', { exact: true }).fill('source-bundle.json');
      const acknowledgement = panel.getByLabel('Include exact source bytes and sensitive metadata in this export', { exact: true });
      const prepare = panel.getByRole('button', { name: 'Prepare source export', exact: true });
      assert.equal(await prepare.getAttribute('aria-disabled'), 'true');
      await acknowledgement.check();
      assert.equal(await prepare.getAttribute('aria-disabled'), 'false');
      await prepared(prepare);
      await captures('source-export-preview');
      await dismiss(prepare);
      await prepared(prepare);
      await page.getByRole('button', { name: 'Confirm this exact write', exact: true }).press('Enter');
      await page.waitForFunction(() => document.getElementById('status').textContent === 'Saved source-bundle.json.');
      await page.locator('#preview-dialog[open]').waitFor({ state: 'hidden' });
      const exported = fs.readFileSync(path.join(project, 'source-bundle.json'));
      assert(exported.length <= 1048429);
      const bundle = JSON.parse(exported);
      assert.equal(bundle.schema_version, 'forge.workspace-index-bundle/3');
      assert.equal(bundle.profile, 'index-and-source-hex');
      assert.equal(bundle.source_content_included, true);
      const index = JSON.parse(fs.readFileSync(path.join(project, 'forge.workspace.json')));
      assert.deepEqual(bundle.index, index);
      assert.equal(bundle.pins.length, index.resources.length);
      assert.equal(bundle.contents.length, index.resources.length);
      for (let number = 0; number < index.resources.length; number++) {
        const registration = index.resources[number];
        const pin = bundle.pins[number];
        const content = bundle.contents[number];
        const actual = fs.readFileSync(path.join(project, registration.path));
        assert.deepEqual(pin, { key: registration.key, sha256: sha(actual), size: actual.length });
        assert.equal(content.key, registration.key);
        assert.equal(content.encoding, 'hex');
        assert(content.chunks.every(chunk => typeof chunk === 'string' && /^[0-9a-f]*$/.test(chunk) && chunk.length % 2 === 0));
        assert.deepEqual(Buffer.from(content.chunks.join(''), 'hex'), actual);
      }
      const download = page.getByRole('button', { name: 'Download committed source bundle', exact: true });
      const pendingDownload = page.waitForEvent('download');
      await download.press('Enter');
      const downloaded = await pendingDownload;
      assert.equal(downloaded.suggestedFilename(), 'forge-workspace-index-and-source-content.json');
      assert(!fs.existsSync(path.join(shared, 'source-bundle.json')));
      await downloaded.saveAs(path.join(shared, 'source-bundle.json'));
      assert.deepEqual(fs.readFileSync(path.join(shared, 'source-bundle.json')), exported);
      const after = inventory(project);
      for (const [name, pin] of Object.entries(before)) assert.deepEqual(after[name], pin);
      assert.deepEqual(Object.keys(after).sort(), [...Object.keys(before), 'source-bundle.json'].sort());
      receipt.export_sha256 = sha(exported);
      receipt.export_bytes = exported.length;
      receipt.download_exact_bytes = true;
      receipt.download_filename = downloaded.suggestedFilename();
    } else if (stage === 'restore') {
      receipt.phase = 'restore-preview';
      const incoming = path.join(shared, 'source-bundle.json');
      await panel.getByLabel('Choose an exact source bundle JSON file', { exact: true }).setInputFiles(incoming);
      await panel.getByLabel('Source restore target index schema', { exact: true }).selectOption('2');
      const prepare = panel.getByRole('button', { name: 'Prepare source restore', exact: true });
      assert.equal(await prepare.getAttribute('aria-disabled'), 'true');
      for (const label of ['I acknowledge replacing the complete project index label and registrations',
        'I acknowledge including exact source bytes and sensitive metadata',
        'I understand the complete preview may create or overwrite project files']) {
        await panel.getByLabel(label, { exact: true }).check();
      }
      const response = page.waitForResponse(value => value.request().method() === 'POST' && new URL(value.url()).pathname === '/api/v2/project/source-bundle-imports');
      await prepared(prepare);
      const actualResponse = await response;
      assert.equal(actualResponse.status(), 200);
      const raw = await actualResponse.body();
      assert(raw.length <= 1024 * 1024);
      const actual = JSON.parse(raw);
      const preview = actual.preview;
      assert.equal(preview.operation_type, 'project-source-restore');
      assert.match(preview.operation_id, /^op_[0-9a-z]{12,80}$/);
      assert.equal(actual.replacement.consumed_file_count, 4);
      assert.deepEqual(actual.replacement.removed_resource_keys, ['old-policy', 'retired']);
      assert.deepEqual(actual.replacement.previous_index, JSON.parse(fs.readFileSync(path.join(project, 'forge.workspace.json'))));
      assert.deepEqual(actual.replacement.proposed_index, JSON.parse(fs.readFileSync(path.join(donor, 'forge.workspace.json'))));
      assert.deepEqual(preview.targets.map(value => value.path), ['policy.md', 'refs/framework.json', 'forge.workspace.json']);
      assert.equal(preview.targets[0].status, 'overwrite');
      assert.equal(preview.targets[1].status, 'create');
      assert.equal(preview.targets[2].status, 'overwrite');
      assert.equal(preview.input_bindings.length, 4);
      assert.deepEqual(preview.input_bindings.map(value => value.path).sort(), ['forge.workspace.json', 'policy.md', 'refs/framework.json', 'retired.md']);
      assert.deepEqual(preview.directories.map(value => value.path), ['refs']);
      for (const target of preview.targets.slice(0, -1)) {
        const expected = donorBefore[target.path];
        assert.equal(target.exact_bytes_sha256, expected.sha256);
        assert.equal(target.size, expected.size);
      }
      assert.equal(preview.targets.at(-1).exact_bytes_sha256, actual.replacement.proposed_index_sha256);
      known = { operation_id: preview.operation_id, exact_manifest_sha256: preview.exact_manifest_sha256,
        targets: preview.targets.map(value => ({ path: value.path, sha256: value.exact_bytes_sha256, size: value.size })) };
      receipt.confirmed_plan = known;
      assert.equal(await page.getByRole('region', { name: 'Previous index complete source restore membership', exact: true }).locator('tbody tr').count(), 2);
      assert.equal(await page.getByRole('region', { name: 'Proposed index complete source restore membership', exact: true }).locator('tbody tr').count(), 2);
      assert.match(await page.locator('#preview-content').textContent(), /Every target in publication order; index last/);
      await captures('source-restore-preview');
      assert.deepEqual(inventory(project), before);
      const exactAck = page.getByLabel('I reviewed every target, input binding, directory and index replacement; restore these exact bytes', { exact: true });
      const confirm = page.getByRole('button', { name: 'Confirm this exact source restore', exact: true });
      assert.equal(await confirm.getAttribute('aria-disabled'), 'true');
      await exactAck.check();
      assert.equal(await confirm.getAttribute('aria-disabled'), 'false');
      await confirm.press('Enter');
      await page.locator('#preview-dialog[open]').waitFor({ state: 'hidden' });
      const row = await succeeded(known.operation_id, known);
      assert.equal(receipt.restore_commit_requests, 1);
      const after = inventory(project);
      for (const target of known.targets) assert.deepEqual(after[target.path], { sha256: target.sha256, size: target.size });
      assert.deepEqual(after['retired.md'], before['retired.md']);
      assert.deepEqual(Object.keys(after).sort(), ['forge.workspace.json', 'policy.md', 'refs/framework.json', 'retired.md']);
      assert.deepEqual(JSON.parse(fs.readFileSync(path.join(project, 'forge.workspace.json'))), actual.replacement.proposed_index);
      await row.scrollIntoViewIfNeeded();
      await captures('source-restore-status', row.locator('[data-source-restore-status]'));
      writeJSON(path.join(shared, 'known-outcome.json'), known);
      receipt.complete_project_effects_verified = true;
      receipt.removed_source_retained = true;
    } else {
      receipt.phase = 'fresh-read-only-lookup';
      known = JSON.parse(fs.readFileSync(path.join(shared, 'known-outcome.json')));
      assert.match(known.operation_id, /^op_[0-9a-z]{12,80}$/);
      assert.equal(await panel.getByRole('button', { name: 'Prepare source export', exact: true }).getAttribute('aria-disabled'), 'true');
      assert.equal(await panel.getByRole('button', { name: 'Prepare source restore', exact: true }).getAttribute('aria-disabled'), 'true');
      await panel.getByLabel('Source restore outcome operation ID', { exact: true }).fill(known.operation_id);
      await panel.getByRole('button', { name: 'Look up source restore outcome', exact: true }).press('Enter');
      const row = await succeeded(known.operation_id, known);
      await row.scrollIntoViewIfNeeded();
      await captures('fresh-read-only-outcome', row.locator('[data-source-restore-status]'));
      assert.equal(receipt.restore_commit_requests, 0);
      assert(!receipt.requests.some(value => /^POST \/api\/v2\/project\/(source-bundle-|bundle-restores)/.test(value)));
      assert.deepEqual(inventory(project), before);
      receipt.readonly_project_unchanged = true;
      receipt.known_operation_id = known.operation_id;
    }

    receipt.phase = 'UI-shutdown';
    await page.locator('#stop').press('Enter');
    await page.locator('#confirm-stop').press('Enter');
    await page.waitForFunction(() => document.getElementById('status').textContent.startsWith('Workspace stopped'));
    await focused(page.locator('#main'), 'stopped-main');
    if (known) {
      const row = page.locator('[data-source-restore-row="' + known.operation_id + '"]');
      assert.equal(await row.count(), 1);
      assert.match(await row.locator('[data-source-restore-status]').textContent(), /Keep this operation ID/);
    }
    await Promise.all([...pendingObservations]);
    assert(!observerError);
    assert.equal(receipt.page_errors, 0);
    assert.equal(receipt.non_loopback_page_requests, 0);
    assert.deepEqual(fs.readFileSync(path.join(root, 'ui/workspace.js')), sourceJS);
    assert.deepEqual(fs.readFileSync(path.join(root, 'ui/workspace.css')), sourceCSS);
    receipt.fixture_after = inventory(project);
    receipt.UI_shutdown_observed = true;
    receipt.source_assets_unchanged = true;
    receipt.status = 'passed';
    receipt.phase = 'complete';
  } catch (error) {
    receipt.error_class = error.constructor.name;
    process.exitCode = 1;
    if (page) {
      try {
        await page.screenshot({ path: path.join(out, 'failure.png'), fullPage: false, timeout: 5000 });
        receipt.screenshots.push('failure.png');
      } catch { receipt.failure_screenshot_observed = false; }
    }
  } finally {
    if (context) {
      try { await context.close(); receipt.context_close_verified = true; }
      catch { receipt.status = 'failed'; receipt.context_close_error = true; process.exitCode = 1; }
    }
    if (browser) {
      try { await browser.close(); receipt.browser_close_verified = true; }
      catch { receipt.status = 'failed'; receipt.browser_close_error = true; process.exitCode = 1; }
    }
    if (!receipt.context_close_verified || !receipt.browser_close_verified) receipt.status = 'failed';
    if (receipt.status !== 'passed') process.exitCode = 1;
    writeJSON(path.join(out, 'receipt.json'), receipt);
    process.stdout.write(JSON.stringify({ status: receipt.status, stage, phase: receipt.phase }) + '\n');
  }
})();
