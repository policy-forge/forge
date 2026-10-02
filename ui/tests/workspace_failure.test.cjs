/** Exercise redacted first-fault diagnostics and actual CJS startup with owned mocks only. */
"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const {errors} = require("playwright");
const {STAGES,AWAIT_STEPS,OPERATORS,ownDataValue,failureRecord,createTracker,reconcileCleanup,publishOutcome} = require("./workspace_failure.cjs");

/** Obtain an actual native assertion error from the requested failing operation. */
function actualAssertion(operation) {
  try { operation(); }
  catch (error) { return error; }
  throw new Error("The negative assertion did not fail");
}

/** Require exactly the public fixed envelope, without retaining private error text. */
function checkEnvelope(record, stage, category, operator = null, step = null) {
  assert.deepEqual(record,{schema_version:"forge.workspace-browser-failure/2",stage,
    category,assertion_operator:operator,await_step:step});
  assert.equal(Object.isFrozen(record),true);
}

/** Classify the four actual native operators used by the maintained browser assertions. */
function nativeAssertionOperators() {
  const probes = [
    [()=>assert.equal(1,2),"strictEqual"],
    [()=>assert.deepEqual({a:1},{a:2}),"deepStrictEqual"],
    [()=>assert.match("private input",/^unmatched$/),"match"],
    [()=>assert(false,"private passphrase"),"=="],
  ];
  assert.deepEqual(OPERATORS,probes.map(probe=>probe[1]));
  for (const [probe,operator] of probes) checkEnvelope(
    failureRecord("metadata-comparison",actualAssertion(probe),errors.TimeoutError),
    "metadata-comparison","assertion",operator);
}

/** Recognize the actual approved Playwright TimeoutError and reject a name-only impostor. */
function actualTimeoutBrand() {
  checkEnvelope(failureRecord("metadata-long-label",new errors.TimeoutError("private DOM"),errors.TimeoutError),
    "metadata-long-label","timeout");
  checkEnvelope(failureRecord("unlock",{name:"TimeoutError",message:"private token"},errors.TimeoutError),
    "unlock","unclassified");
}

/** A name-only assertion impostor cannot add an operator or assertion category. */
function fakeAssertionBrand() {
  checkEnvelope(failureRecord("unlock",{name:"AssertionError",operator:"strictEqual"},errors.TimeoutError),
    "unlock","unclassified");
}

/** A real assertion with an unknown operator remains classified with a null operator. */
function unknownOperator() {
  const error = actualAssertion(()=>assert.notEqual(1,1));
  checkEnvelope(failureRecord("metadata-preview",error,errors.TimeoutError),"metadata-preview","assertion");
}

/** Operator accessors and inherited properties never execute or supply diagnostic data. */
function operatorAccessor() {
  let reads = 0;
  const error = actualAssertion(()=>assert(false));
  Object.defineProperty(error,"operator",{get(){ reads++;throw new Error("private getter"); }});
  checkEnvelope(failureRecord("resource-authoring",error,errors.TimeoutError),"resource-authoring","assertion");
  assert.equal(reads,0);
  assert.equal(ownDataValue(Object.create({operator:"=="}),"operator"),undefined);
  assert.equal(ownDataValue(null,"operator"),undefined);
}

/** Proxy reflection faults produce a fixed unclassified result and cannot interrupt cleanup. */
function proxyReflectionFault() {
  const error = new Proxy({}, {getPrototypeOf(){throw new Error("private proxy");}});
  checkEnvelope(failureRecord("browser-setup",error,errors.TimeoutError),"browser-setup","unclassified");
  const branded = new Proxy(actualAssertion(()=>assert(false)),{
    getOwnPropertyDescriptor(){throw new Error("private descriptor");},
  });
  checkEnvelope(failureRecord("browser-setup",branded,errors.TimeoutError),"browser-setup","unclassified");
}

/** Primitive thrown values disclose no data and receive only the unclassified category. */
function primitiveErrors() {
  for (const error of [null,undefined,"private path",42,Symbol("private secret")])
    checkEnvelope(failureRecord("storage-checks",error,errors.TimeoutError),"storage-checks","unclassified");
}

/** The serialized fixed diagnostic excludes every private value from an actual error. */
function privateDataExcluded() {
  const secret = "/private/project/secret?token=passphrase";
  const error = actualAssertion(()=>assert.equal(secret,"<private DOM>"));
  const value = JSON.stringify(failureRecord("metadata-file-selection",error,errors.TimeoutError));
  for (const privateValue of [secret,"<private DOM>","message","stack","actual","expected"])
    assert.equal(value.includes(privateValue),false);
}

/** The first fault and stage survive a later stage and even an uninspectable error proxy. */
function firstFaultSticky() {
  const tracker = createTracker(errors.TimeoutError);
  assert.equal(tracker.current(),null);assert.equal(tracker.hasFailure(),false);
  tracker.setStage("metadata-comparison");
  const first = tracker.capture(actualAssertion(()=>assert.equal(1,2)));
  tracker.setStage("browser-cleanup");
  const later = new Proxy({}, {getPrototypeOf(){throw new Error("Must not inspect later error");}});
  assert.equal(tracker.capture(later),first);
  assert.equal(tracker.hasFailure(),true);
  checkEnvelope(tracker.current(),"metadata-comparison","assertion","strictEqual");
}

/** Unsupported stages never enter the public protocol or mutate a valid tracker stage. */
function stageAllowlist() {
  const tracker = createTracker(errors.TimeoutError);
  assert.equal(STAGES.length,23);assert.equal(new Set(STAGES).size,23);
  assert.throws(()=>tracker.setStage("private project label"),TypeError);
  assert.throws(()=>failureRecord("private path",new Error(),errors.TimeoutError),TypeError);
  tracker.capture(new Error("private"));
  checkEnvelope(tracker.current(),"browser-setup","unclassified");
}

/** Both owned closes run when the first close fails, and a prior main fault stays primary. */
async function bothClosesAndPrimaryFault() {
  const tracker = createTracker(errors.TimeoutError);const calls=[];
  tracker.setStage("metadata-download");tracker.capture(new errors.TimeoutError("private timeout"));
  await reconcileCleanup(tracker,{async close(){calls.push("context");throw new Error("first close");}},
    {async close(){calls.push("browser");throw new Error("second close");}});
  assert.deepEqual(calls,["context","browser"]);
  checkEnvelope(tracker.current(),"metadata-download","timeout");
}

/** A cleanup-only fault revokes success, even though both independent closes are attempted. */
async function cleanupOnlyFault() {
  const tracker = createTracker(errors.TimeoutError);const calls=[];
  await reconcileCleanup(tracker,{async close(){calls.push("context");throw new Error("private");}},
    {async close(){calls.push("browser");}});
  const output=[];
  assert.equal(publishOutcome(tracker,{mode:"writable"},value=>output.push(value)),false);
  assert.deepEqual(calls,["context","browser"]);assert.equal(output.length,1);
  checkEnvelope(output[0],"browser-cleanup","unclassified");
}

/** Success publishes its exact original object only after clean reconciliation of both closes. */
async function cleanSuccessPublication() {
  const tracker = createTracker(errors.TimeoutError);const calls=[];const observed={mode:"read-only"};
  await reconcileCleanup(tracker,{async close(){calls.push("context");}},
    {async close(){calls.push("browser");}});
  const output=[];
  assert.equal(publishOutcome(tracker,observed,value=>{assert.deepEqual(calls,["context","browser"]);output.push(value);}),true);
  assert.deepEqual(output,[observed]);assert.equal(output[0],observed);
  const empty=createTracker(errors.TimeoutError);await reconcileCleanup(empty,null,null);
  assert.equal(empty.current(),null);
}

/** Missing success observation produces one fixed failure instead of JSON null or a pass. */
function missingSuccessObservation() {
  const tracker=createTracker(errors.TimeoutError);const output=[];
  assert.equal(publishOutcome(tracker,null,value=>output.push(value)),false);
  assert.equal(output.length,1);checkEnvelope(output[0],"browser-setup","unclassified");
}

/** Execute the unchanged interaction script in a VM with mock startup ownership only. */
async function mockedStartup(chromium,url="http://127.0.0.1:1234") {
  const stdout=[];const stderr=[];const processMock={argv:["node","workspace.cjs",url,"writable"],env:{},exitCode:0};
  const source=fs.readFileSync(path.join(__dirname,"workspace.cjs"),"utf8");
  const sandbox={__dirname,process:processMock,URL,Buffer,
    console:{log(value){stdout.push(value);},error(...values){stderr.push(values);}},
    require(name){
      if(name==="playwright")return {chromium,errors};
      if(name==="./workspace_failure.cjs")return require(name);
      return require(name);
    }};
  await vm.runInNewContext(source,sandbox,{filename:"workspace.cjs",timeout:1000});
  assert.equal(stdout.length,1);assert.equal(processMock.exitCode,1);
  return JSON.parse(stdout[0]);
}

/** Actual CJS startup failure emits one redacted record before any browser is owned. */
async function actualCjsLaunchFailure() {
  const record=await mockedStartup({async launch(){throw new errors.TimeoutError("private launch arguments");}});
  assert.deepEqual(record,{schema_version:"forge.workspace-browser-failure/2",stage:"browser-setup",
    category:"timeout",assertion_operator:null,await_step:null});
}

/** Actual CJS retains its URL assertion and never launches when that assertion fails. */
async function actualCjsUrlFailure() {
  let launches=0;
  const record=await mockedStartup({async launch(){launches++;throw new Error();}},"https://private.example");
  assert.equal(launches,0);
  assert.deepEqual(record,{schema_version:"forge.workspace-browser-failure/2",stage:"browser-setup",
    category:"assertion",assertion_operator:"match",await_step:null});
}

/** Actual CJS first-fault publication follows both closes even when private diagnostics fail. */
async function actualCjsOwnedCleanup() {
  const calls=[];
  const record=await mockedStartup({async launch(){return {
    async newContext(){return {async newPage(){throw new errors.TimeoutError("private page");},
      async close(){calls.push("context");throw new Error("private context");}};},
    async close(){calls.push("browser");throw new Error("private browser");},
  };}});
  assert.deepEqual(calls,["context","browser"]);
  assert.deepEqual(record,{schema_version:"forge.workspace-browser-failure/2",stage:"browser-setup",
    category:"timeout",assertion_operator:null,await_step:null});
}

/** Each fixed metadata refresh await retains its exact step on an actual branded fault. */
function refreshAwaitSteps() {
  assert.deepEqual(AWAIT_STEPS,["prepare","activate","response","ready","acknowledgment","download-state","focus","dialog"]);
  for (const step of AWAIT_STEPS) {
    const tracker=createTracker(errors.TimeoutError);
    tracker.setStage("metadata-refresh");tracker.setAwaitStep(step);
    tracker.capture(new errors.TimeoutError("private refresh context"));
    checkEnvelope(tracker.current(),"metadata-refresh","timeout",null,step);
  }
}

/** Invalid or misplaced step values never enter the protocol and never invoke private coercion. */
function refreshStepAllowlist() {
  const tracker=createTracker(errors.TimeoutError);let coerced=0;
  const privateValue={toString(){coerced++;throw new Error("private value");}};
  assert.throws(()=>tracker.setAwaitStep("ready"),TypeError);
  tracker.setStage("metadata-refresh");tracker.setAwaitStep("response");
  for (const value of [null,undefined,"private path",privateValue]) {
    assert.throws(()=>tracker.setAwaitStep(value),TypeError);
    if (value !== undefined && value !== null)
      assert.throws(()=>failureRecord("metadata-refresh",new Error(),errors.TimeoutError,value),TypeError);
  }
  assert.throws(()=>failureRecord("unlock",new Error(),errors.TimeoutError,"ready"),TypeError);
  assert.equal(coerced,0);
  tracker.capture(new Error("private"));checkEnvelope(tracker.current(),"metadata-refresh","unclassified",null,"response");
}

/** A main-stage transition clears a stale step, including reentry into metadata refresh. */
function refreshStepReset() {
  for (const stage of ["storage-checks","metadata-refresh"]) {
    const tracker=createTracker(errors.TimeoutError);tracker.setStage("metadata-refresh");
    tracker.setAwaitStep("dialog");tracker.setStage(stage);
    tracker.capture(new Error("private"));checkEnvelope(tracker.current(),stage,"unclassified");
  }
}

/** The first actual assertion keeps its await step through both failing closes and publication. */
async function refreshStepStickyCleanup() {
  const tracker=createTracker(errors.TimeoutError);const calls=[];const output=[];
  tracker.setStage("metadata-refresh");tracker.setAwaitStep("acknowledgment");
  const first=tracker.capture(actualAssertion(()=>assert.equal(true,false)));
  await reconcileCleanup(tracker,{async close(){calls.push("context");throw new Error("private");}},
    {async close(){calls.push("browser");throw new Error("private");}});
  assert.equal(publishOutcome(tracker,{mode:"writable"},value=>output.push(value)),false);
  assert.deepEqual(calls,["context","browser"]);assert.equal(output.length,1);assert.equal(output[0],first);
  checkEnvelope(output[0],"metadata-refresh","assertion","strictEqual","acknowledgment");
}

/** Actual post-refresh continuation clears the completed dialog step before a private screenshot can fail. */
async function postRefreshScreenshotHasNoStaleStep() {
  const source=fs.readFileSync(path.join(__dirname,"workspace.cjs"),"utf8");
  const start=source.indexOf("const metadataConsumerObservation=await verifyMetadataConsumer();");
  const finish=source.indexOf('failureTracker.setStage("final-reflow");',start);
  assert(start>=0&&finish>start);
  const tracker=createTracker(errors.TimeoutError);let screenshots=0;
  const sandbox={failureTracker:tracker,process:{env:{FORGE_TEST_SCREENSHOT:"private screenshot path"}},
    async verifyMetadataConsumer(){tracker.setStage("metadata-refresh");tracker.setAwaitStep("dialog");return {};},
    page:{async screenshot(){screenshots++;throw new errors.TimeoutError("private screenshot fault");}}};
  try {await vm.runInNewContext(`(async()=>{${source.slice(start,finish)}})()`,sandbox,{timeout:1000});}
  catch(error){tracker.capture(error);}
  assert.equal(screenshots,1);
  checkEnvelope(tracker.current(),"metadata-refresh","timeout");
}

for (const control of [nativeAssertionOperators,actualTimeoutBrand,fakeAssertionBrand,unknownOperator,
  operatorAccessor,proxyReflectionFault,primitiveErrors,privateDataExcluded,firstFaultSticky,stageAllowlist,
  bothClosesAndPrimaryFault,cleanupOnlyFault,cleanSuccessPublication,missingSuccessObservation,
  actualCjsLaunchFailure,actualCjsUrlFailure,actualCjsOwnedCleanup,refreshAwaitSteps,refreshStepAllowlist,refreshStepReset,refreshStepStickyCleanup,postRefreshScreenshotHasNoStaleStep]) test(control.name,control);
