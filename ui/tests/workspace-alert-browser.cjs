// Real installed-Chrome verification of informational due/overdue review alerts in the workspace owner queue.
// Synthetic project only; no privileged API calls, no injected state, no governed-byte mutation.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");
const [origin, project, out, root] = process.argv.slice(2);
assert.match(origin, /^http:\/\/127\.0\.0\.1:[0-9]+$/);
assert(project && out && root);
const { chromium } = require(path.join(root, "ui/node_modules/playwright"));

function sha(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
/** Pin only the explicitly authored project files, refusing symbolic aliases. */
function files() {
  const result = {};
  for (const name of fs.readdirSync(project).sort()) {
    const full = path.join(project, name);
    assert(!fs.lstatSync(full).isSymbolicLink(), "Fixture must not contain symbolic aliases");
    if (fs.statSync(full).isFile()) result[name] = sha(fs.readFileSync(full));
  }
  return result;
}

(async () => {
  const receipt = { schema: "forge.review-alert-browser/1", origin, phases: [], screenshots: [] };
  let browser;
  try {
    const options = { headless: true, args: ["--disable-background-networking"] };
    if (process.env.FORGE_TEST_BROWSER_EXECUTABLE) options.executablePath = process.env.FORGE_TEST_BROWSER_EXECUTABLE;
    browser = await chromium.launch(options);
    const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, serviceWorkers: "block" });
    const page = await context.newPage();
    page.setDefaultTimeout(20000);
    const nonLoopback = [];
    const lifecycleWrites = [];
    page.on("request", request => {
      const url = new URL(request.url());
      if (url.origin !== origin) nonLoopback.push(request.url());
      if (url.pathname.startsWith("/api/v2/lifecycle") && request.method() !== "GET") lifecycleWrites.push(request.url());
    });
    const before = files();
    await page.goto(origin, { waitUntil: "domcontentloaded" });
    assert.equal(await page.locator('meta[name="forge-api-major"]').getAttribute("content"), "2");
    const negotiated = await page.locator('meta[name="forge-api-contract-version"]').getAttribute("content");
    assert(["2.1.0", "2.2.0", "2.3.0", "2.4.0"].includes(negotiated));
    receipt.contract_version = negotiated;
    await page.getByLabel("Workspace passphrase", { exact: true }).fill("synthetic review alert browser passphrase 065");
    await page.getByRole("button", { name: "Unlock workspace", exact: true }).click();
    await page.locator("#navigation").waitFor();
    receipt.phases.push("unlock");
    // Navigate to the lifecycle inspection view.
    await page.getByRole("button", { name: "Lifecycle & Impact", exact: true }).click();
    await page.getByLabel("Explicit lifecycle review date").waitFor();
    receipt.phases.push("lifecycle-view");
    // Apply an explicit date one day past the 2026-10-12 review: the placement is overdue.
    await page.getByLabel("Explicit lifecycle review date").fill("2026-10-13");
    await page.getByRole("button", { name: "Apply lifecycle date and filters", exact: true }).click();
    const queue = page.locator('[data-inspection-page="queue"]');
    await queue.waitFor();
    await queue.locator("[data-review-alerts]").waitFor();
    const alertSummary = await queue.locator("[data-review-alerts]").textContent();
    assert.match(alertSummary, /1 overdue and 0 due-soon review placements on this page/);
    assert.match(alertSummary, /do not change review state or imply approval/);
    // The placement table carries a Review alert column with the overdue badge.
    const queueText = await queue.textContent();
    assert.match(queueText, /Review alert/);
    assert.match(queueText, /overdue/);
    receipt.phases.push("overdue-alert");
    await page.screenshot({ path: path.join(out, "overdue-alert.png") });
    receipt.screenshots.push("overdue-alert.png");
    // A date inside the due-soon window (review 2026-10-12, window 7 days) yields a due-soon alert.
    await page.getByLabel("Explicit lifecycle review date").fill("2026-10-10");
    await page.getByRole("button", { name: "Apply lifecycle date and filters", exact: true }).click();
    await queue.locator("[data-review-alerts]").filter({ hasText: "0 overdue and 1 due-soon review placements" }).waitFor();
    const dueSoonSummary = await queue.locator("[data-review-alerts]").textContent();
    assert.match(dueSoonSummary, /0 overdue and 1 due-soon review placements on this page/);
    receipt.phases.push("due-soon-alert");
    await page.screenshot({ path: path.join(out, "due-soon-alert.png") });
    receipt.screenshots.push("due-soon-alert.png");
    // A date well before the window reports no alerts.
    await page.getByLabel("Explicit lifecycle review date").fill("2026-09-01");
    await page.getByRole("button", { name: "Apply lifecycle date and filters", exact: true }).click();
    await queue.locator("[data-review-alerts]").filter({ hasText: "No due or overdue reviews" }).waitFor();
    receipt.phases.push("clean-portfolio");
    await page.screenshot({ path: path.join(out, "clean-portfolio.png") });
    receipt.screenshots.push("clean-portfolio.png");
    // Alerts are read-only: no outbound fetch, no lifecycle write, no governed-byte change.
    assert.deepEqual(nonLoopback, [], "Non-loopback request observed");
    assert.deepEqual(lifecycleWrites, [], "Lifecycle write request observed");
    assert.deepEqual(files(), before, "Governed project bytes changed");
    receipt.phases.push("invariants");
    fs.writeFileSync(path.join(out, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
    console.log(JSON.stringify({ status: "ok", phases: receipt.phases }));
  } catch (error) {
    receipt.status = "error";
    receipt.error = String(error && error.message ? error.message : error);
    fs.writeFileSync(path.join(out, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
    console.error(receipt.error);
    process.exitCode = 1;
  } finally {
    if (browser) await browser.close();
  }
})();