// Real embedded UI interactions: no privileged API calls or injected project state.
const {chromium,errors:browserErrors}=require("playwright");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const {createHash}=require("node:crypto");
const {createTracker,reconcileCleanup,publishOutcome}=require("./workspace_failure.cjs");
const failureTracker=createTracker(browserErrors.TimeoutError);
let outcomePublished=false;
(async()=>{
 let browser=null;let context=null;let page=null;let observation=null;
 try {
 const url=process.argv[2];const readOnly=process.argv[3]==="read-only";const longMetadata=process.env.FORGE_TEST_LONG_METADATA==="1";
 assert.match(url,/^http:\/\/127\.0\.0\.1:[0-9]+$/);
 const options={headless:true,args:["--disable-background-networking"]};
 if(process.env.FORGE_TEST_BROWSER_EXECUTABLE)options.executablePath=process.env.FORGE_TEST_BROWSER_EXECUTABLE;
 browser=await chromium.launch(options);
 context=await browser.newContext({viewport:{width:1280,height:900},serviceWorkers:"block"});
 page=await context.newPage();page.setDefaultTimeout(20000);
 const violations=[];const calls=new Set();const errors=[];let provenanceReads=0;let focusChecks=0;let embeddedAssetSha256;let embeddedStyleSha256;let inputBorderContrast;let conversionRequests=0;let unlockRequests=0;const reflowChecks=[];const syntheticFaults=[];const measuredOperationResponses=[];const measuredOperationTasks=new Set();
  page.on("request",request=>{
    const pathname=new URL(request.url()).pathname;
    if(pathname==="/api/v1/provenance/entries")provenanceReads++;
    if(pathname==="/api/v1/conversions"&&request.method()==="POST")conversionRequests++;
    if(pathname==="/api/v1/session/unlock"&&request.method()==="POST")unlockRequests++;
  });
 const contract=fs.readFileSync(path.join(__dirname,"../../docs/api/forge-workspace-v1.openapi.yaml"),"utf8");
 const documented=[];let routePath;
 for(const line of contract.split("\n")) {
   const pathMatch=line.match(/^  (\/api\/v1\/[^:]+):$/);if(pathMatch)routePath=pathMatch[1];
   const method=line.match(/^    (get|post|put|delete|patch):$/);
   if(method&&routePath)documented.push({method:method[1].toUpperCase(),path:new RegExp("^"+routePath.replace(/\{[^}]+\}/g,"[^/]+")+"$")});
 }
 await context.route("**/*",async route=>{
   const request=route.request();const target=new URL(request.url());
   if(target.origin!==url){violations.push("non-loopback request");await route.abort();return;}
   if(target.pathname.startsWith("/api/")){
     if(!documented.some(operation=>operation.method===request.method()&&operation.path.test(target.pathname)))violations.push("undocumented API route: "+target.pathname);
     calls.add(request.method()+" "+target.pathname.replace(/(?:op|prev|res|prov|ex)_[a-z0-9]+/g,"{id}"));
   }else if(target.pathname!=="/"&&!/^\/assets\/[a-f0-9]{64}\.(js|css)$/.test(target.pathname))violations.push("undocumented asset");
   await route.continue();
 });
 /** Retain only actual operation counter facts, without logging response bodies or authority. */
 async function observeMeasuredOperation(response) {
   const pathname=new URL(response.url()).pathname;
   if(!response.ok()||response.headers()["x-forge-consumer-probe"]||
      !/^\/api\/v1\/(?:operations\/op_[0-9a-z]+(?:\/cancellation)?|conversions|applicability\/analyses|mapping\/builds|exports)$/.test(pathname))return;
   const value=await response.json();
   const progress=value.progress;
   if(typeof value.operation_id!=="string"||!/^op_[0-9a-z]{12,80}$/.test(value.operation_id)||
      !progress||!Number.isSafeInteger(progress.completed_items)||progress.completed_items<0||
      !Number.isSafeInteger(progress.total_items)||progress.total_items<0)return;
   measuredOperationResponses.push({id:value.operation_id,kind:value.kind,state:value.state,
     completed:progress.completed_items,total:progress.total_items});
 }
 /** Track asynchronous real-response reads so their facts settle before comparison. */
 function queueMeasuredOperation(response) {
   const task=observeMeasuredOperation(response).catch(()=>errors.push("Measured operation response could not be retained."));
   measuredOperationTasks.add(task);
   void task.finally(()=>measuredOperationTasks.delete(task));
 }
 page.on("response",queueMeasuredOperation);
 page.on("pageerror",error=>errors.push(error.message));
   failureTracker.setStage("asset-binding");
   const assetResponse=page.waitForResponse(response=>/^\/assets\/[a-f0-9]{64}\.js$/.test(new URL(response.url()).pathname));
   const styleResponse=page.waitForResponse(response=>/^\/assets\/[a-f0-9]{64}\.css$/.test(new URL(response.url()).pathname));
   await page.goto(url);
   const servedAsset=await (await assetResponse).body();
   const expectedAsset=fs.readFileSync(path.join(__dirname,"../workspace.js"));
   assert.deepEqual(servedAsset,expectedAsset,"the server must embed the exact candidate JavaScript");
   embeddedAssetSha256=createHash("sha256").update(servedAsset).digest("hex");
   const servedStyle=await (await styleResponse).body();
   assert.deepEqual(servedStyle,fs.readFileSync(path.join(__dirname,"../workspace.css")),"the server must embed the exact candidate stylesheet");
   embeddedStyleSha256=createHash("sha256").update(servedStyle).digest("hex");
   // Test-only DOM observation reads statuses; it supplies no project or operation state.
   await page.evaluate(()=>{
     const observations=[];const seen=new Set();
     window.__forgeCapturedRegistrationObservations=observations;
     /** Retain distinct rendered capture facts without moving focus or recording private text. */
     function retainRenderedCaptureFacts() {
       for(const status of document.querySelectorAll("[data-operation-status]")) {
         const match=status.textContent.match(/^Operation (?:pending|running)\. (?:Cancellation requested; awaiting terminal state\. )?Captured registrations: ([0-9]+) of ([0-9]+) reported\.$/);
         if(!match)continue;
         const id=status.closest("[data-operation-id]")?.dataset.operationId;
         const completed=Number(match[1]);const total=Number(match[2]);
         if(!id||!Number.isSafeInteger(completed)||!Number.isSafeInteger(total))continue;
         const key=`${id}|${completed}|${total}`;
         if(seen.has(key))continue;
         seen.add(key);observations.push({id,completed,total});
       }
     }
     const observer=new MutationObserver(retainRenderedCaptureFacts);
     observer.observe(document.getElementById("main"),{childList:true,characterData:true,subtree:true});
     window.__forgeCapturedRegistrationObserver=observer;
   });
   // These measured colors qualify only this fixture's input boundary, not full AA acceptance.
   failureTracker.setStage("input-style-binding");
   const inputColors=await page.getByLabel("Workspace passphrase").evaluate(input=>{
     const style=getComputedStyle(input);
     let ancestor=input.parentElement;let background="rgb(255, 255, 255)";
     while(ancestor){
       const candidate=getComputedStyle(ancestor).backgroundColor;
       if(candidate!=="rgba(0, 0, 0, 0)"&&candidate!=="transparent"){background=candidate;break;}
       ancestor=ancestor.parentElement;
     }
     return {border:style.borderTopColor,fill:style.backgroundColor,ancestor:background};
   });
   /** Calculate relative luminance of an opaque computed CSS RGB color. */
   const luminance=color=>{
     const match=color.match(/^rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)$/);
     assert(match,"Expected opaque computed RGB color, received "+color);
     const channels=match.slice(1).map(value=>Number(value)/255).map(value=>value<=0.04045?value/12.92:((value+0.055)/1.055)**2.4);
     return channels[0]*0.2126+channels[1]*0.7152+channels[2]*0.0722;
   };
   /** Compare an actual border color with its actual adjacent fixture surface. */
   const contrast=(foreground,background)=>{
     const values=[luminance(foreground),luminance(background)].sort((a,b)=>b-a);
     return (values[0]+0.05)/(values[1]+0.05);
   };
   inputBorderContrast={...inputColors,againstFill:contrast(inputColors.border,inputColors.fill),againstAncestor:contrast(inputColors.border,inputColors.ancestor)};
   assert(inputBorderContrast.againstFill>=3,"the input border must contrast with its fill by at least 3:1");
   assert(inputBorderContrast.againstAncestor>=3,"the input border must contrast with its adjacent background by at least 3:1");
   /** Activate a native control with Enter after explicitly setting keyboard focus. */
   const activate=async locator=>{await locator.focus();await locator.press("Enter");};
   /** Observe exact focus after async work without assigning a target to satisfy the assertion. */
   const focused=async locator=>{await page.waitForFunction(node=>node===document.activeElement,await locator.elementHandle());assert(await locator.evaluate(node=>node===document.activeElement));focusChecks++;};
   failureTracker.setStage("unlock");
   const credential=page.getByLabel("Workspace passphrase");
   await focused(credential);
   assert.equal(await page.locator("#status").textContent(),"Workspace locked — passphrase required.");
   assert.equal(await credential.getAttribute("aria-describedby"),"status");
   // This documented429 is a delayed synthetic UI fault, not authentic server timing evidence.
   const throttleMessage="Too many attempts — retry available in 2 seconds. Wait, then retry. If repeated, stop and relaunch the workspace from the terminal.";
   let releaseThrottle;
   const throttleGate=new Promise(resolve=>{releaseThrottle=resolve;});
   /** Hold one closed Error-envelope response until pending keyboard behavior is observed. */
   const delayThrottle=async route=>{await throttleGate;await route.fulfill({status:429,contentType:"application/json",body:JSON.stringify({code:"unlock-throttled",message:throttleMessage,retryable:true})});};
   await page.route("**/api/v1/session/unlock",delayThrottle,{times:1});
   await credential.fill("synthetic browser verification passphrase 062");
   const pendingUnlock=page.waitForRequest(request=>new URL(request.url()).pathname==="/api/v1/session/unlock"&&request.method()==="POST");
   await credential.press("Enter");await pendingUnlock;
   await page.waitForFunction(()=>document.getElementById("status").textContent==="Unlocking workspace…");
   await focused(credential);
   assert.equal(await page.locator("#unlock-form").getAttribute("aria-busy"),"true");
   const unlockButton=page.getByRole("button",{name:"Unlock workspace",exact:true});
   assert.equal(await unlockButton.getAttribute("aria-disabled"),"true");
   const nativeUnlockState=await unlockButton.evaluate(control=>({disabled:control.disabled,connected:control.isConnected}));
   assert.deepEqual(nativeUnlockState,{disabled:false,connected:true},"Pending focus must remain on a connected native control");
   await credential.press("Enter");
   // Observe beyond the client's60ms request spacing while the injected reply is still held.
   await page.waitForTimeout(150);assert.equal(unlockRequests,1,"Pending Enter must not duplicate unlock transport");
   releaseThrottle();
   await page.waitForFunction(message=>document.getElementById("status").textContent===message,throttleMessage);
   await focused(credential);
   assert.equal(await credential.inputValue(),"");
   assert.equal(await page.locator("#error").isVisible(),false);
   assert.equal(await page.locator("#unlock-form").getAttribute("aria-busy"),"false");
   assert.equal(await unlockButton.getAttribute("aria-disabled"),"false");
   syntheticFaults.push({kind:"unlock-throttled",status:429,message:throttleMessage,duplicateSubmitObservationMs:150,scope:"delayed documented UI fault; no authentic server delay qualification"});
   await credential.fill("synthetic browser verification passphrase 062");
   await credential.press("Enter");
   await page.getByRole("heading",{name:"Overview",exact:true}).waitFor();
   // Visible shell headings precede async unlock completion; observe readiness without setting focus.
   await page.waitForFunction(()=>document.activeElement===document.getElementById("main")
     && !document.getElementById("view").inert
     && document.getElementById("status").textContent==="Project state loaded.");
   // Keyboard activation exercises the native button/dialog contracts without a pointer.
   /** Verify heading focus or the declared prerequisite error before a manifest exists. */
   const navigate=async(name,initialInventoryError)=>{
     await activate(page.getByRole("navigation").getByRole("button",{name,exact:true}));
     await page.getByRole("heading",{name,exact:true}).waitFor();
     if(initialInventoryError){
       const error=page.locator("[data-page-error]");await error.waitFor({state:"visible"});
       assert.equal(await error.textContent(),"validation-failed: "+initialInventoryError+" Retryable: false. No results could be loaded.");
       await page.getByLabel("New decision manifest path within project",{exact:true}).waitFor();
       await focused(error);
     }else await focused(page.locator("#view-title"));
   };
   /** Choose the explicit destructive dialog action, then wait for native close. */
   const discard=async()=>{await activate(page.getByRole("dialog").getByRole("button",{name:"Discard edits",exact:true}));await page.getByRole("dialog").waitFor({state:"hidden"});};
   /** Locate the persistent framework results region rather than a replaceable table. */
   const controlResults=()=>page.locator('[data-page-results][aria-label="Framework control inventory"]');
   /** Scope status, controls and recoverable errors to the framework inventory section. */
   const controlSection=()=>page.locator("section").filter({has:controlResults()}).first();
   /** Wait for committed pagination state, then verify exact keyboard focus and row count. */
   const controlPage=async(matching,total,index,rows)=>{
     const summary=controlSection().locator("[data-page-status]");
     const text=matching+" matching items of "+total+" total · Page "+index+".";
     await page.waitForFunction(({node,text})=>node.textContent===text,{node:await summary.elementHandle(),text});
     assert.equal(await summary.textContent(),text);
     assert.equal(await summary.getAttribute("role"),"status");
     assert.equal(await summary.getAttribute("aria-live"),"polite");
     assert.equal(await summary.getAttribute("aria-atomic"),"true");
     await page.waitForFunction(node=>node.getAttribute("aria-busy")==="false",await controlResults().elementHandle());
     assert.equal(await controlResults().locator("tbody tr").count(),rows);
     await focused(controlResults());
   };
   /** Match only real documented inventory reads, separating filtered data and metadata. */
   const controlsResponse=(classification,pageSize)=>page.waitForResponse(response=>{
     const target=new URL(response.url());
     return response.request().method()==="GET"&&target.pathname==="/api/v1/applicability/controls"
       &&target.searchParams.get("classification")===(classification||null)
       &&target.searchParams.get("page_size")===String(pageSize)&&response.ok();
   });
   /** Apply a keyboard filter and compare the real filtered/unfiltered response versions. */
   const filterControls=async(classification,matching,total)=>{
     const filtered=controlsResponse(classification,50);
     const unfiltered=classification?controlsResponse("",1):null;
     await page.getByLabel("Classification",{exact:true}).selectOption(classification);
     await activate(controlSection().getByRole("button",{name:"Apply filters",exact:true}));
     const value=await (await filtered).json();
     assert.equal(value.page.total_matching,matching);
     assert.match(value.resource_version,/^[a-f0-9]{64}$/);
     if(unfiltered){
       const baseline=await (await unfiltered).json();
       assert.equal(baseline.page.total_matching,total);
       assert.equal(value.resource_version,baseline.resource_version,"filtered and unfiltered counts must belong to one captured project version");
     }else assert.equal(matching,total);
     await controlPage(matching,total,1,Math.min(matching,50));
   };
   failureTracker.setStage("navigation-recovery");
   // A failed destination read must leave the focused summary intact, then recover normally.
   await page.route("**/api/v1/resources?*",route=>route.fulfill({status:500,contentType:"application/json",body:JSON.stringify({code:"internal-error",message:"Synthetic navigation failure.",retryable:true})}),{times:1});
   await activate(page.getByRole("navigation").getByRole("button",{name:"Policies & Artifacts",exact:true}));
   await page.locator("#error").waitFor({state:"visible"});assert.match(await page.locator("#error").textContent(),/Synthetic navigation failure/);await focused(page.locator("#error"));
   /** Confirm with native Enter, then observe queued close and the exact current saved focus target. */
   const confirm=async(refreshFailure)=>{
     const dialog=page.getByRole("dialog");
     await activate(dialog.getByRole("button",{name:"Confirm this exact write",exact:true}));
     await dialog.waitFor({state:"hidden"});
     await page.waitForFunction(()=>document.getElementById("status").textContent.startsWith("Saved "));
     if(refreshFailure){
       const error=page.locator("#error");await error.waitFor({state:"visible"});
       assert.match(await error.textContent(),/write was saved, but the view could not be refreshed/);
       assert((await error.textContent()).includes(refreshFailure));await focused(error);
     }else await focused(page.locator("#view-title"));
   };
   failureTracker.setStage("resource-authoring");
   await navigate("Policies & Artifacts");
   if(readOnly){assert.equal(await page.getByRole("button",{name:"Preview registration"}).count(),0);}
   else {
     const register=async(file,role,key)=>{await page.getByLabel("Resource role",{exact:true}).selectOption(role);await page.getByLabel("Project-relative file path",{exact:true}).fill(file);await page.getByLabel("Stable resource key",{exact:true}).fill(key);await page.getByRole("button",{name:"Preview registration",exact:true}).click();await confirm();};
     await register("policy.md","policy-source","policy");
     // Inspect must preserve registration values and invoker focus for Keep editing/Escape.
     const registrationPath=page.getByLabel("Project-relative file path",{exact:true});
     const registrationKey=page.getByLabel("Stable resource key",{exact:true});
     const inspectPolicy=page.getByRole("button",{name:"Inspect policy",exact:true});
     await registrationPath.fill("unconfirmed-policy.md");await registrationKey.fill("unconfirmed-key");
     const beforeInspect=provenanceReads;
     await activate(inspectPolicy);await page.getByRole("dialog").waitFor();await focused(page.getByRole("heading",{name:"Discard unconfirmed edits?",exact:true}));
     await activate(page.getByRole("dialog").getByRole("button",{name:"Keep editing",exact:true}));await page.getByRole("dialog").waitFor({state:"hidden"});
     assert.equal(await registrationPath.inputValue(),"unconfirmed-policy.md");assert.equal(await registrationKey.inputValue(),"unconfirmed-key");await focused(inspectPolicy);assert.equal(provenanceReads,beforeInspect);
     await activate(inspectPolicy);await page.getByRole("dialog").waitFor();await page.keyboard.press("Escape");await page.getByRole("dialog").waitFor({state:"hidden"});
     assert.equal(await registrationPath.inputValue(),"unconfirmed-policy.md");assert.equal(await registrationKey.inputValue(),"unconfirmed-key");await focused(inspectPolicy);assert.equal(provenanceReads,beforeInspect);
     // A failed approved inspection keeps unsaved values, error focus, and the next discard gate.
     await page.route("**/api/v1/provenance/entries?*",route=>route.fulfill({status:500,contentType:"application/json",body:JSON.stringify({code:"internal-error",message:"Synthetic provenance failure.",retryable:true})}),{times:1});
     await activate(inspectPolicy);await discard();await page.locator("#error").waitFor({state:"visible"});await focused(page.locator("#error"));assert.match(await page.locator("#error").textContent(),/Synthetic provenance failure/);
     assert.equal(await registrationPath.inputValue(),"unconfirmed-policy.md");assert.equal(await registrationKey.inputValue(),"unconfirmed-key");
     await activate(inspectPolicy);await page.getByRole("dialog").waitFor();await discard();await page.getByRole("heading",{name:"Provenance references",exact:true}).waitFor();await focused(page.locator("#view-title"));
     await navigate("Policies & Artifacts");assert.equal(await page.getByRole("dialog").count(),0,"confirmed discard must clear the old dirty state");
     // Inject documented transport failures without publishing or replacing project data.
     await page.getByLabel("Resource role",{exact:true}).selectOption("oscal-catalog-artifact");await page.getByLabel("Project-relative file path",{exact:true}).fill("framework.json");await page.getByLabel("Stable resource key",{exact:true}).fill("framework");await page.getByRole("button",{name:"Preview registration",exact:true}).click();
     await page.route("**/api/v1/effects/commits",route=>route.fulfill({status:409,contentType:"application/json",body:JSON.stringify({code:"receipt-expired",message:"The preview expired. Prepare a new preview.",retryable:false})}),{times:1});
     await activate(page.getByRole("dialog").getByRole("button",{name:"Confirm this exact write",exact:true}));const alert=page.getByRole("dialog").getByRole("alert");await alert.waitFor();assert.match(await alert.textContent(),/receipt-expired/);assert(await alert.evaluate(node=>node===document.activeElement));
     await page.getByRole("dialog").getByRole("button",{name:"Keep editing",exact:true}).click();assert.equal(await page.getByLabel("Project-relative file path",{exact:true}).inputValue(),"framework.json");
     await page.getByRole("button",{name:"Preview registration",exact:true}).click();await page.getByRole("dialog").waitFor();assert.equal(await page.getByRole("dialog").getByRole("alert").count(),0,"a new preview must not retain the previous confirmation error");
     await page.route("**/api/v1/resources?*",route=>route.fulfill({status:500,contentType:"application/json",body:JSON.stringify({code:"internal-error",message:"Synthetic refresh failure.",retryable:true})}),{times:1});
     await confirm("Synthetic refresh failure.");await page.locator("#error").waitFor({state:"visible"});assert.match(await page.locator("#error").textContent(),/write was saved.*Synthetic refresh failure/);assert(await page.locator("#error").evaluate(node=>node===document.activeElement));await page.getByRole("button",{name:"Refresh",exact:true}).click();await page.getByLabel("Output model").waitFor();
     await page.getByLabel("Output model").selectOption("oscal-catalog");await page.getByLabel("Output project-relative path").fill("converted.json");
     failureTracker.setStage("conversion-recovery");
     // Use a real preparation and real known ID; only its first status read is fault injected.
     const preparationsBefore=conversionRequests;
     const conversionReply=page.waitForResponse(response=>response.request().method()==="POST"&&new URL(response.url()).pathname==="/api/v1/conversions"&&response.ok());
     /** Lose one documented polling GET without substituting an operation state or result. */
     const failOperationRead=async route=>{
       if(route.request().method()==="GET"){
         await route.fulfill({status:500,contentType:"application/json",body:JSON.stringify({code:"internal-error",message:"Synthetic operation read failure.",retryable:true})});
         await page.unroute("**/api/v1/operations/op_*",failOperationRead);
       }else await route.continue();
     };
     await page.route("**/api/v1/operations/op_*",failOperationRead);
     await activate(page.getByRole("button",{name:"Prepare conversion",exact:true}));
     const prepared=await (await conversionReply).json();
     assert.match(prepared.operation_id,/^op_[0-9a-z]{12,80}$/);assert.equal(prepared.kind,"conversion");
     const operation=page.locator('article[data-operation-id="'+prepared.operation_id+'"]');
     const operationStatus=operation.locator("[data-operation-status]");
     await operation.getByRole("button",{name:"Check operation status",exact:true}).waitFor();
     assert.match(await operationStatus.textContent(),/Operation outcome is unknown/);
     assert.match(await operation.locator("[data-operation-error]").textContent(),/Synthetic operation read failure/);
     assert.equal(await operationStatus.getAttribute("role"),"status");assert.equal(await operationStatus.getAttribute("aria-live"),"polite");assert.equal(await operationStatus.getAttribute("aria-atomic"),"true");
     assert.equal(await page.locator("#view-title").textContent(),"Policies & Artifacts");
     assert.equal(await page.getByLabel("Output project-relative path").inputValue(),"converted.json");
     assert.equal(await page.getByRole("dialog").count(),0);
     const operationHandle=await operation.elementHandle();
     await activate(page.getByRole("navigation").getByRole("button",{name:"Trace & Reports",exact:true}));
     await page.getByRole("dialog").waitFor();await discard();await page.getByRole("heading",{name:"Trace & Reports",exact:true}).waitFor();await focused(page.locator("#view-title"));
     assert(await operation.evaluate((node,original)=>node===original,operationHandle),"navigation must retain the session-owned operation row");
     assert.equal(await operation.evaluate(node=>!!node.closest("#view")||!!node.closest("#status")),false);
     const checkOperation=operation.getByRole("button",{name:"Check operation status",exact:true});
     const recovered=page.waitForResponse(async response=>{
       if(response.request().method()!=="GET"||new URL(response.url()).pathname!=="/api/v1/operations/"+prepared.operation_id||!response.ok())return false;
       return (await response.json()).state==="succeeded";
     });
     await activate(checkOperation);
     const actualResult=await (await recovered).json();
     assert.equal(actualResult.operation_id,prepared.operation_id);
     await operation.getByRole("button",{name:"Review prepared write",exact:true}).waitFor();
     assert.equal(await operationStatus.textContent(),"Operation succeeded. Preparation is ready for review; no write has been confirmed.");
     assert.equal(conversionRequests,preparationsBefore+1,"known-ID recovery must not repeat the preparation POST");
     assert.equal(await page.getByRole("dialog").count(),0,"background completion must not open a confirmation dialog");
     assert.equal(await page.locator("#view-title").textContent(),"Trace & Reports");
     await focused(checkOperation);
     await activate(operation.getByRole("button",{name:"Review prepared write",exact:true}));
     await page.getByRole("dialog").waitFor();await focused(page.getByRole("heading",{name:"Review proposed write",exact:true}));await confirm();
     await navigate("Policies & Artifacts");
     await register("converted.json","oscal-catalog-artifact","converted");
     failureTracker.setStage("framework-workflow");
     await navigate("Framework Scope","Register one valid applicability manifest and its dependencies.");await page.getByLabel("Framework Catalog",{exact:true}).selectOption({label:"framework · framework.json"});await page.getByLabel("New decision manifest path within project").fill("scope.json");await page.getByRole("button",{name:"Preview initial scope manifest"}).click();await confirm("Register one valid applicability manifest and its dependencies.");
     await navigate("Policies & Artifacts");await register("scope.json","applicability-manifest","scope");
     await navigate("Framework Scope");
     // The launcher supplies 53 authored controls; ordinary navigation retains heading focus.
     await controlResults().waitFor();
     const firstPageRows=await controlResults().locator("tbody tr").allTextContents();
     assert.equal(firstPageRows.length,50);
     assert.equal(await controlSection().locator("[data-page-status]").textContent(),"53 matching items of 53 total · Page 1.");
     const nextPage=controlSection().getByRole("button",{name:"Next page",exact:true,includeHidden:true});
     const previousPage=controlSection().getByRole("button",{name:"Previous page",exact:true,includeHidden:true});
     const nextHandle=await nextPage.elementHandle();const previousHandle=await previousPage.elementHandle();
     await activate(nextPage);await controlPage(53,53,2,3);
     const lastPageRows=await controlResults().locator("tbody tr").allTextContents();
     assert.equal(new Set([...firstPageRows,...lastPageRows]).size,53,"forward traversal must preserve every authored control exactly once");
     assert(await nextPage.evaluate((node,original)=>node===original,nextHandle),"Next page must retain its DOM identity");
     assert(await previousPage.evaluate((node,original)=>node===original,previousHandle),"Previous page must retain its DOM identity");
     await activate(previousPage);await controlPage(53,53,1,50);
     assert.deepEqual(await controlResults().locator("tbody tr").allTextContents(),firstPageRows);
     // A documented read failure must retain the committed page and focus an inline alert.
     /** Lose one cursor read while leaving other legitimate inventory requests untouched. */
     const failNext=async route=>{
       const target=new URL(route.request().url());
       if(target.searchParams.get("page_size")==="50"&&target.searchParams.has("cursor")){
         await route.fulfill({status:500,contentType:"application/json",body:JSON.stringify({code:"internal-error",message:"Synthetic next-page failure.",retryable:true})});
         await page.unroute("**/api/v1/applicability/controls?*",failNext);
       }else await route.continue();
     };
     await page.route("**/api/v1/applicability/controls?*",failNext);
     await activate(nextPage);
     const pageError=controlSection().locator("[data-page-error]");
     await pageError.waitFor({state:"visible"});assert.match(await pageError.textContent(),/Synthetic next-page failure/);await focused(pageError);
     assert.equal(await page.locator("#view-title").textContent(),"Framework Scope");
     assert.deepEqual(await controlResults().locator("tbody tr").allTextContents(),firstPageRows);
     assert.equal(await controlSection().locator("[data-page-status]").textContent(),"Framework control inventory: Results could not be loaded. Previous results: 53 matching items of 53 total · Page 1.");
     await activate(controlSection().getByRole("button",{name:"Retry page",exact:true}));await controlPage(53,53,2,3);
     await activate(previousPage);await controlPage(53,53,1,50);
     await filterControls("under-review",53,53);
     await filterControls("not-applicable",0,53);
     assert.equal(await controlResults().getByText("No matching items.",{exact:true}).count(),1);
     await filterControls("",53,53);
     // Overview's count card deliberately moves focus to the destination results.
     await navigate("Overview");
     const reviewReply=page.waitForResponse(response=>new URL(response.url()).pathname==="/api/v1/review-queue/items"&&response.ok());
     await activate(page.getByRole("button",{name:/Open review items/}));
     await page.getByRole("heading",{name:"Review Queue",exact:true}).waitFor();
     const reviewValue=await (await reviewReply).json();
     assert.equal(reviewValue.page.total_matching,53);
     const reviewResults=page.locator('[data-page-results][aria-label="Items requiring human review"]');
     await focused(reviewResults);
     assert.equal(await page.locator("section").filter({has:reviewResults}).first().locator("[data-page-status]").textContent(),"53 matching items of 53 total · Page 1.");
     await navigate("Framework Scope");
     // Inspecting scope evidence must not erase the unsaved full decision document.
     const scopeDocument=page.getByLabel("Decision manifest (JSON)",{exact:true});
     const unsavedScope=(await scopeDocument.inputValue())+"\n ";await scopeDocument.fill(unsavedScope);
     const inspectScope=page.getByRole("button",{name:"Inspect framework-a",exact:true}).first();const beforeScopeInspect=provenanceReads;
     await activate(inspectScope);await page.getByRole("dialog").waitFor();await page.keyboard.press("Escape");await page.getByRole("dialog").waitFor({state:"hidden"});
     assert.equal(await scopeDocument.inputValue(),unsavedScope);await focused(inspectScope);assert.equal(provenanceReads,beforeScopeInspect);
     failureTracker.setStage("decision-authoring");
     await page.getByLabel("Control to review").selectOption("framework-a");await page.getByLabel("Explicit decision").selectOption("applicable");await page.getByLabel("Decision reviewer key").fill("reviewer");await page.getByLabel("Decision reviewer name").fill("Synthetic reviewer <script>" );await page.getByLabel("Decision review time (RFC3339)").fill("2026-09-10T00:00:00Z");await page.getByLabel("Decision rationale").fill("Explicit synthetic scope review");
     await page.getByRole("button",{name:"Apply decision to unsaved manifest"}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();
     // One real committed decision changes the matching count without changing total scope.
     await filterControls("under-review",52,53);
     await filterControls("",53,53);
     await page.getByRole("button",{name:"Analyze committed scope decisions"}).click();await confirm();
     await navigate("Mappings","The selected domain inputs are missing, ambiguous, invalid, or stale.");
     await page.getByLabel("Framework Catalog",{exact:true}).selectOption({label:"framework · framework.json"});await page.getByLabel("Policy Catalog",{exact:true}).selectOption({label:"converted · converted.json"});await page.getByLabel("New decision manifest path within project").fill("mapping-manifest.json");await page.getByLabel("Mapping review scope").selectOption("control-only");
     for(const [label,value] of [["Stable collection key","mapping"],["Mapping collection title","Synthetic mapping"],["Document version","1"],["Review time (RFC3339, including timezone)","2026-09-10T00:00:00Z"],["Reviewer key","reviewer"],["Asserted reviewer name","Synthetic mapping reviewer"],["Intended use and limitations","Synthetic explicit review only"],["Stable relationship key","reviewed-none"],["Relationship rationale","Explicit absence of a positive relationship"]])await page.getByLabel(label,{exact:true}).fill(value);
     await page.getByLabel("Review matching rationale").selectOption("semantic");await page.getByRole("button",{name:"Load selected Catalog subjects"}).click();
     await page.getByLabel("Reviewed policy subject").selectOption({index:1});await page.getByLabel("Reviewed framework subject").selectOption({label:"Control: framework-a"});await page.getByLabel("Reviewed relationship",{exact:true}).selectOption("no-relationship");
     await page.getByRole("button",{name:"Preview initial mapping manifest"}).click();await confirm("The selected domain inputs are missing, ambiguous, invalid, or stale.");
     await navigate("Policies & Artifacts");await register("mapping-manifest.json","mapping-collection","mapping");await navigate("Mappings");
     await page.getByLabel("Relationship to review").selectOption("reviewed-none");await page.getByLabel("Mapping review rationale").fill("Explicit updated rationale");await page.getByRole("button",{name:"Apply relationship to unsaved manifest"}).click();await page.getByRole("button",{name:"Validate decisions"}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();
     await page.getByLabel("New relationship key",{exact:true}).fill("reviewed-additional");await page.getByLabel("New relationship policy subject",{exact:true}).selectOption({index:1});await page.getByLabel("New relationship framework subject",{exact:true}).selectOption({label:"framework-b"});await page.getByLabel("New relationship type",{exact:true}).selectOption("equivalent-to");await page.getByLabel("New relationship reviewer",{exact:true}).selectOption("reviewer");await page.getByLabel("New relationship review time (RFC3339)",{exact:true}).fill("2026-09-10T00:00:00Z");await page.getByLabel("New relationship rationale",{exact:true}).fill("Explicit additional synthetic review");await page.getByRole("button",{name:"Add relationship to unsaved manifest",exact:true}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();
     await page.getByRole("button",{name:"Rebuild committed mapping collection"}).click();await confirm();
     await navigate("Policies & Artifacts");await register("mapping-collection.json","mapping-collection","built-mapping");
     await navigate("Framework Scope");await page.getByLabel("Reviewed mapping collection",{exact:true}).selectOption({label:"built-mapping · mapping-collection.json"});await page.getByRole("button",{name:"Link collection to unsaved scope",exact:true}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();await page.getByRole("button",{name:"Analyze committed scope decisions"}).click();await confirm();
     assert(await page.getByRole("cell",{name:"applicable-reviewed-no-relationship",exact:true}).count()>0);
     await navigate("Trace & Reports");
     failureTracker.setStage("trace-export");
     const reportTarget=page.getByLabel("Report destination within project");const traceConverted=page.getByRole("button",{name:"Trace converted",exact:true});
     await reportTarget.fill("unconfirmed-report.html");const beforeTrace=provenanceReads;
     await activate(traceConverted);await page.getByRole("dialog").waitFor();await activate(page.getByRole("dialog").getByRole("button",{name:"Keep editing",exact:true}));await page.getByRole("dialog").waitFor({state:"hidden"});
     assert.equal(await reportTarget.inputValue(),"unconfirmed-report.html");await focused(traceConverted);assert.equal(provenanceReads,beforeTrace);
     await activate(traceConverted);await discard();await page.getByRole("heading",{name:"Provenance references",exact:true}).waitFor();await focused(page.locator("#view-title"));
     await navigate("Trace & Reports");await page.getByLabel("Report",{exact:true}).selectOption("trace");await page.getByLabel("Report destination within project").fill("trace.html");await page.getByRole("button",{name:"Prepare export"}).click();await confirm();
     // Continue from post-close heading using native Tab; do not reset focus for this assertion.
     await page.keyboard.press("Tab");await focused(page.getByRole("button",{name:"Refresh",exact:true}));
     await page.keyboard.press("Tab");await focused(page.getByRole("button",{name:"Download committed redacted report",exact:true}));
     const download=page.waitForEvent("download");await page.keyboard.press("Enter");assert.equal((await download).suggestedFilename(),"forge-redacted-report.html");
   }
    /** Exercise real documented metadata reads and a local download in either session mode. */
    async function verifyMetadataConsumer() {
      failureTracker.setStage("metadata-preview");
      await navigate("Trace & Reports");
      const panel=page.locator("[data-bundle-panel]");
      const preview=panel.getByRole("button",{name:"Preview metadata",exact:true});
      const acknowledgment=page.getByLabel("I understand that labels, resource keys, paths and hashes can reveal project information.",{exact:true});
      const downloadButton=panel.getByRole("button",{name:"Download metadata bundle",exact:true});
      const file=page.getByLabel("Choose a metadata bundle JSON file",{exact:true});
      const compare=panel.getByRole("button",{name:"Compare registered fingerprints",exact:true});
      const globalStatus=await page.locator("#status").textContent();
      const metadataReflow=[];
      /** Measure actual global overflow while permitting table-region scroll, then restore viewport and focus. */
      async function measureMetadataLongContent(stage) {
        const previousViewport=page.viewportSize();const previousFocus=await page.locator(":focus").elementHandle();
        assert(previousViewport);assert(previousFocus);
        try {
          for(const width of [640,320]) {
            await page.setViewportSize({width,height:900});
            const observation=await page.evaluate(stage=>({stage,viewportWidth:window.innerWidth,
              documentWidth:document.documentElement.scrollWidth,bodyWidth:document.body.scrollWidth,
              pageWidth:Math.max(document.documentElement.scrollWidth,document.body.scrollWidth),
              tableRegions:[...document.querySelectorAll("[data-bundle-panel] .table-wrap")].map(node=>({
                caption:node.getAttribute("aria-label"),clientWidth:node.clientWidth,scrollWidth:node.scrollWidth,
                left:node.getBoundingClientRect().left,right:node.getBoundingClientRect().right})),
              metadataTextScalars:[...document.querySelectorAll("[data-bundle-metadata] p")].map(node=>[...node.textContent].length),
              comparisonStatusScalars:[...document.querySelectorAll("[data-bundle-comparison-status]")].map(node=>[...node.textContent].length)}),stage);
            metadataReflow.push(observation);
            console.error("Metadata long-content reflow observation:",JSON.stringify(observation));
          }
        } finally {
          await page.setViewportSize(previousViewport);
          // Measurement explicitly restores the captured native control; subsequent Tab probes start there.
          if(await previousFocus.evaluate(node=>node.isConnected))await previousFocus.focus();
        }
      }
      assert.equal(await acknowledgment.isChecked(),false);
      assert.equal(await downloadButton.getAttribute("aria-disabled"),"true");
      const previewReply=page.waitForResponse(response=>response.request().method()==="GET"&&new URL(response.url()).pathname==="/api/v1/project/bundle-preview"&&response.ok());
      await activate(preview);
      const observed=await (await previewReply).json();
      await page.waitForFunction(({node,count})=>node.textContent===`Metadata preview: ${count} registered resources. No project file was written.`,
        {node:await panel.locator("[data-bundle-preview-status]").elementHandle(),count:observed.bundle.pins.length});
      assert.equal(observed.source_content_included,false);
      assert.equal(observed.bundle.content_profile,"index-and-hashes");
      assert.equal(observed.bundle.pins.length,observed.bundle.index.resources.length);
      assert.equal(await panel.getByRole("table",{name:"Complete registered bundle metadata",exact:true}).locator("tbody tr").count(),observed.bundle.pins.length);
      await focused(preview);
      failureTracker.setStage("metadata-long-label");
      let projectLabelObservation;
      if(longMetadata) {
        const label=observed.bundle.index.label;
        assert.equal(label,"<script>"+"L".repeat(192));
        const metadataLabel=panel.locator("[data-bundle-metadata] p").first();
        assert.equal(await metadataLabel.textContent(),"Project: "+label);
        assert.equal(await panel.locator("script").count(),0,"The actual server label must remain literal text");
        projectLabelObservation={scalarCount:[...label].length,utf8Bytes:Buffer.byteLength(label),
          literalScriptPrefix:label.startsWith("<script>"),unbrokenSuffixScalars:[...label.slice(8)].length,
          exactAuthoredLabel:label==="<script>"+"L".repeat(192),renderedAsText:true};
        console.error("Metadata long-content project-label observation:",JSON.stringify(projectLabelObservation));
        await measureMetadataLongContent("after-preview");await focused(preview);
      }
      failureTracker.setStage("metadata-download");
      // Observe native Tab order after the asynchronous preview, without assigning these targets.
      await page.keyboard.press("Tab");await focused(panel.getByRole("region",{name:"Complete registered bundle metadata",exact:true}));
      await page.keyboard.press("Tab");await focused(acknowledgment);await page.keyboard.press("Space");
      assert.equal(await acknowledgment.isChecked(),true);assert.equal(await downloadButton.getAttribute("aria-disabled"),"false");
      await page.keyboard.press("Tab");await focused(downloadButton);
      const localDownload=page.waitForEvent("download");await page.keyboard.press("Enter");const downloaded=await localDownload;
      assert.equal(downloaded.suggestedFilename(),"forge-workspace-index-and-hashes.json");
      const downloadedPath=await downloaded.path();assert(downloadedPath);const bytes=fs.readFileSync(downloadedPath);
      assert.deepEqual(bytes,Buffer.from(JSON.stringify(observed.bundle)),"Local download must contain the complete observed bundle alone");
      assert(bytes.length<=1024*1024);await focused(downloadButton);
      failureTracker.setStage("metadata-file-selection");
      await page.keyboard.press("Tab");await focused(file);
      // setInputFiles supplies an external synthetic File through the native input; no OS chooser claim.
      const chosenName=longMetadata?"N".repeat(180)+".json":"observed-metadata.json";
      await file.setInputFiles({name:chosenName,mimeType:"application/json",buffer:bytes});
      let chosenFileObservation;
      if(longMetadata) {
        chosenFileObservation=await file.evaluate(node=>{const chosen=node.files[0];return {
          name:chosen.name,scalarCount:[...chosen.name].length,byteLength:chosen.size,type:chosen.type};});
        assert.equal(chosenFileObservation.name,chosenName);assert.equal(chosenFileObservation.scalarCount,185);
        assert.equal(chosenFileObservation.byteLength,bytes.length);
        assert.equal(await panel.locator("[data-bundle-comparison-status]").textContent(),`Chosen file: ${chosenName} (${bytes.length} bytes). Compare explicitly.`);
        console.error("Metadata long-content chosen-file observation:",JSON.stringify(chosenFileObservation));
        await measureMetadataLongContent("after-file-chosen-before-compare");await focused(file);
        const overflow=metadataReflow.filter(value=>value.pageWidth>value.viewportWidth);
        assert.deepEqual(overflow,[],"Actual metadata long content overflowed the global page: "+JSON.stringify(overflow));
      }
      failureTracker.setStage("metadata-comparison");
      await page.keyboard.press("Tab");await focused(compare);
      const comparisonReply=page.waitForResponse(response=>response.request().method()==="POST"&&new URL(response.url()).pathname==="/api/v1/project/bundle-verifications"&&response.ok());
      await page.keyboard.press("Enter");const actualComparisonReply=await comparisonReply;const comparison=await actualComparisonReply.json();
      const raw=actualComparisonReply.request().postDataBuffer();assert(raw);assert.deepEqual(raw,Buffer.concat([Buffer.from('{"bundle":'),bytes,Buffer.from('}')]));
      assert.equal(raw.length,bytes.length+11);assert(raw.length<=1024*1024);
      assert.equal(actualComparisonReply.request().headers()["idempotency-key"],undefined);
      await page.waitForFunction(({node,count})=>node.textContent.startsWith(`Registered fingerprints: ${count} matched,`),
        {node:await panel.locator("[data-bundle-comparison-status]").elementHandle(),count:observed.bundle.pins.length});
      assert.equal(comparison.scope,"registered-fingerprints-only");assert.equal(comparison.source_content_included,false);
      assert.equal(comparison.state,"matched");assert.equal(comparison.expected_resources,observed.bundle.pins.length);
      assert.equal(comparison.matched_resources,comparison.expected_resources);assert.equal(comparison.unregistered_resources,0);assert.equal(comparison.mismatched_resources,0);
      assert.equal(comparison.expected_index_matches_current,true);assert.equal(comparison.current_only_resources,0);
      assert.equal(await panel.getByRole("table",{name:"Complete expected fingerprint comparison",exact:true}).locator("tbody tr").count(),comparison.expected_resources);
      await focused(compare);
      failureTracker.setStage("metadata-duplicate");
      // A duplicate raw key remains a strict server-parser rejection, never a client-side projection.
      const duplicate=Buffer.concat([Buffer.from('{"schema_version":"forge.workspace-index-bundle/1",'),bytes.subarray(1)]);
      await file.setInputFiles({name:"duplicate-metadata.json",mimeType:"application/json",buffer:duplicate});
      const rejectedReply=page.waitForResponse(response=>response.request().method()==="POST"&&new URL(response.url()).pathname==="/api/v1/project/bundle-verifications"&&response.status()===400);
      await activate(compare);const rejected=await rejectedReply;assert.equal((await rejected.json()).code,"invalid-request");
      assert.deepEqual(rejected.request().postDataBuffer(),Buffer.concat([Buffer.from('{"bundle":'),duplicate,Buffer.from('}')]));
      const localError=panel.locator("[data-bundle-comparison-error]");await localError.waitFor({state:"visible"});await focused(localError);
      assert.equal(await page.locator("#error").isVisible(),false);assert.equal(await page.locator("#status").textContent(),globalStatus);
      failureTracker.setStage("metadata-refresh");
      // A fresh actual preview revokes the earlier disclosure acknowledgment, even in read-only mode.
      const refreshed=page.waitForResponse(response=>response.request().method()==="GET"&&new URL(response.url()).pathname==="/api/v1/project/bundle-preview"&&response.ok());
      await activate(preview);await refreshed;
      await page.waitForFunction(node=>node.getAttribute("aria-disabled")==="false",await preview.elementHandle());
      assert.equal(await acknowledgment.isChecked(),false);assert.equal(await downloadButton.getAttribute("aria-disabled"),"true");await focused(preview);
      assert.equal(await page.getByRole("dialog").count(),0);
      return {previewRegistrations:observed.bundle.pins.length,comparisonExpected:comparison.expected_resources,
        localDownloadBytes:bytes.length,localDownloadSha256:createHash("sha256").update(bytes).digest("hex"),rawEnvelopeBytes:raw.length,
        strictDuplicateStatus:400,longContent:longMetadata?{projectLabelObservation,chosenFileObservation,metadataReflow,
          initialIndexFixture:readOnly?"same two registered resources, authored long label":"preauthored empty index with long label before existing UI registration",
          qualification:"Actual server label/native File and global-page measurements; table-region scroll allowed; viewport/focus explicitly restored; scoped reflow observation, not AT/WCAG or full acceptance"}:undefined,
        scope:"actual GET/POST and local download; synthetic external File via native input; no OS chooser, writable import or full S6 acceptance"};
    }
    const metadataConsumerObservation=await verifyMetadataConsumer();
   if(process.env.FORGE_TEST_SCREENSHOT)await page.screenshot({path:process.env.FORGE_TEST_SCREENSHOT,fullPage:true});
   failureTracker.setStage("final-reflow");
   for(const width of [640,320]){
     await page.setViewportSize({width,height:900});
     const observation=await page.evaluate(()=>({viewportWidth:window.innerWidth,pageWidth:document.documentElement.scrollWidth,operations:[...document.querySelectorAll("#operation-region article")].map(node=>{
       const heading=node.querySelector("h2");const rowBox=node.getBoundingClientRect();const headingBox=heading.getBoundingClientRect();
       return {id:node.dataset.operationId,heading:heading.textContent,headingWithinRow:headingBox.left>=rowBox.left&&headingBox.right<=rowBox.right};
     })}));
     assert(observation.pageWidth<=observation.viewportWidth,"Page reflow failed: "+JSON.stringify(observation));
     for(const row of observation.operations){assert.equal(row.heading,"Operation "+row.id);assert(row.headingWithinRow,"Operation heading overflowed its row: "+JSON.stringify(row));}
     reflowChecks.push(observation);
   }
   failureTracker.setStage("storage-checks");
   assert.deepEqual(await context.cookies(),[]);
   assert.deepEqual(await page.evaluate(()=>[localStorage.length,sessionStorage.length]),[0,0]);
   failureTracker.setStage("session-shutdown");
   await page.getByRole("button",{name:"Stop workspace",exact:true}).first().click();
   await page.getByRole("dialog").getByRole("button",{name:"Stop workspace",exact:true}).click();
   await page.getByText("Stopped",{exact:true}).waitFor();
   await Promise.all([...measuredOperationTasks]);
   failureTracker.setStage("counter-correlation");
   const renderedCaptureFacts=await page.evaluate(()=>{
     window.__forgeCapturedRegistrationObserver.disconnect();
     return window.__forgeCapturedRegistrationObservations;
   });
   const responseFacts=new Set(measuredOperationResponses.map(value=>`${value.id}|${value.completed}|${value.total}`));
   assert(renderedCaptureFacts.every(value=>responseFacts.has(`${value.id}|${value.completed}|${value.total}`)),
     "Every rendered captured-registration count must match actual API counter facts");
   const renderedFacts=new Set(renderedCaptureFacts.map(value=>`${value.id}|${value.completed}|${value.total}`));
   const captureConsumerObservation={actualCounterResponses:measuredOperationResponses.length,
     distinctRenderedCounterFacts:renderedCaptureFacts.length,
     responseFactsWithoutRenderedObservation:measuredOperationResponses.filter(value=>!renderedFacts.has(`${value.id}|${value.completed}|${value.total}`)).length,
     qualification:renderedCaptureFacts.length?"actual_api_and_rendered_counter_correlation_only":"no_transient_capture_counter_render_observed"};
   failureTracker.setStage("request-page-errors");
   assert.deepEqual(violations,[]);assert.deepEqual(errors,[]);
   observation={mode:readOnly?"read-only":"writable",browserVersion:browser.version(),embeddedAssetSha256,embeddedStyleSha256,inputBorderContrast,focusChecks,reflowChecks,syntheticFaults,captureConsumerObservation,metadataConsumerObservation,unlockRequests,documentedRequests:[...calls].sort(),nonLoopbackRequests:0,pageErrors:0};
 } catch(error){
   failureTracker.capture(error);process.exitCode=1;
   try {

   console.error(error.stack || error.message);console.error("Error summary:",JSON.stringify({visible:await page.locator("#error").isVisible(),text:await page.locator("#error").textContent()}));
   console.error("Focus state:",JSON.stringify(await page.evaluate(()=>({active:document.activeElement?.outerHTML,error:document.getElementById("error")?.outerHTML,viewInert:document.getElementById("view")?.inert,dialogs:[...document.querySelectorAll("dialog[open]")].map(node=>node.outerHTML)}))));
   if(page!==null)await page.screenshot({path:process.env.FORGE_TEST_SCREENSHOT||"/tmp/forge-workspace-browser-failure.png",fullPage:true});
   } catch(diagnosticError){failureTracker.capture(diagnosticError);}
 } finally {await reconcileCleanup(failureTracker,context,browser);}
 outcomePublished=true;
 if(!publishOutcome(failureTracker,observation,value=>console.log(JSON.stringify(value))))process.exitCode=1;
})().catch(error=>{
 failureTracker.capture(error);process.exitCode=1;
 if(!outcomePublished){outcomePublished=true;console.log(JSON.stringify(failureTracker.current()));}
});
