/** Build small fixed browser diagnostics without retaining error messages, DOM or input values. */
"use strict";
const assert = require("node:assert/strict");
const STAGES = Object.freeze([
  "browser-setup", "asset-binding", "input-style-binding", "unlock",
  "navigation-recovery", "resource-authoring", "conversion-recovery",
  "framework-workflow", "decision-authoring", "trace-export", "metadata-preview",
  "metadata-long-label", "metadata-download", "metadata-file-selection",
  "metadata-comparison", "metadata-duplicate", "metadata-refresh", "final-reflow",
  "storage-checks", "session-shutdown", "counter-correlation", "request-page-errors",
  "browser-cleanup",
]);
const OPERATORS = Object.freeze(["strictEqual", "deepStrictEqual", "match", "=="]);

/** Read an own data property without invoking an accessor or walking a private error prototype. */
function ownDataValue(object, key) {
  if ((typeof object !== "object" && typeof object !== "function") || object === null) return undefined;
  const descriptor = Object.getOwnPropertyDescriptor(object, key);
  return descriptor && Object.hasOwn(descriptor, "value") ? descriptor.value : undefined;
}

/** Classify a branded error using fixed enums; unknown assertion operators stay null. */
function failureRecord(stage, error, TimeoutError) {
  if (!STAGES.includes(stage)) throw new TypeError("Unsupported browser diagnostic stage");
  let category = "unclassified";
  let assertionOperator = null;
  try {
    if (error instanceof assert.AssertionError) {
      category = "assertion";
      const operator = ownDataValue(error, "operator");
      if (OPERATORS.includes(operator)) assertionOperator = operator;
    } else if (typeof TimeoutError === "function" && error instanceof TimeoutError) {
      category = "timeout";
    }
  } catch {
    // Reflection or proxy faults have no authority to bypass cleanup or expose data.
    category = "unclassified";
    assertionOperator = null;
  }
  return Object.freeze({schema_version: "forge.workspace-browser-failure/1", stage,
    category, assertion_operator: assertionOperator});
}

/** Keep the first main fault and its stage when later diagnostics or cleanup also fail. */
function createTracker(TimeoutError) {
  let stage = "browser-setup";
  let first = null;
  /** Change only a fixed main-flow stage; background observations supply no stage values. */
  function setStage(value) {
    if (!STAGES.includes(value)) throw new TypeError("Unsupported browser diagnostic stage");
    stage = value;
  }
  /** Record only the first fault, ignoring later error objects without inspecting them. */
  function capture(error) {
    if (first === null) first = failureRecord(stage, error, TimeoutError);
    return first;
  }
  /** Return the immutable first diagnostic or null without exposing an error object. */
  function current() { return first; }
  /** Report whether success publication must be revoked. */
  function hasFailure() { return first !== null; }
  return Object.freeze({setStage, capture, current, hasFailure});
}

/** Attempt both owned browser closes independently; a cleanup fault never replaces an earlier fault. */
async function reconcileCleanup(tracker, context, browser) {
  tracker.setStage("browser-cleanup");
  for (const owned of [context, browser]) {
    if (owned === null) continue;
    try { await owned.close(); }
    catch (error) { tracker.capture(error); }
  }
}


/** Publish one reconciled outcome; missing success data remains a fixed failure. */
function publishOutcome(tracker, observation, publish) {
  if (observation === null && !tracker.hasFailure()) tracker.capture(new TypeError("Missing browser observation"));
  publish(tracker.current() || observation);
  return !tracker.hasFailure();
}

module.exports = Object.freeze({STAGES, OPERATORS, ownDataValue, failureRecord, createTracker, reconcileCleanup, publishOutcome});
