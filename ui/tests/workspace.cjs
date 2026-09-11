// Real embedded UI interactions: no privileged API calls or injected project state.
const {chromium}=require("playwright");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
(async()=>{
 const url=process.argv[2];const readOnly=process.argv[3]==="read-only";
 assert.match(url,/^http:\/\/127\.0\.0\.1:[0-9]+$/);
 const options={headless:true,args:["--disable-background-networking"]};
 if(process.env.FORGE_TEST_BROWSER_EXECUTABLE)options.executablePath=process.env.FORGE_TEST_BROWSER_EXECUTABLE;
 const browser=await chromium.launch(options);
 const context=await browser.newContext({viewport:{width:1280,height:900},serviceWorkers:"block"});
 const page=await context.newPage();page.setDefaultTimeout(20000);
 const violations=[];const calls=new Set();const errors=[];
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
 page.on("pageerror",error=>errors.push(error.message));
 try {
   await page.goto(url);
   await page.getByLabel("Workspace passphrase").fill("synthetic browser verification passphrase 062");
   await page.getByRole("button",{name:"Unlock workspace",exact:true}).click();
   await page.getByRole("heading",{name:"Overview",exact:true}).waitFor();
   const navigate=async name=>{await page.getByRole("navigation").getByRole("button",{name,exact:true}).click();await page.getByRole("heading",{name,exact:true}).waitFor();};
   const confirm=async()=>{await page.getByRole("dialog").getByRole("button",{name:"Confirm this exact write"}).click();await page.getByRole("dialog").waitFor({state:"hidden"});};
   await navigate("Policies & Artifacts");
   if(readOnly){assert.equal(await page.getByRole("button",{name:"Preview registration"}).count(),0);}
   else {
     const register=async(file,role,key)=>{await page.getByLabel("Resource role",{exact:true}).selectOption(role);await page.getByLabel("Project-relative file path",{exact:true}).fill(file);await page.getByLabel("Stable resource key",{exact:true}).fill(key);await page.getByRole("button",{name:"Preview registration",exact:true}).click();await confirm();};
     await register("policy.md","policy-source","policy");
     // Inject documented transport failures without publishing or replacing project data.
     await page.getByLabel("Resource role",{exact:true}).selectOption("oscal-catalog-artifact");await page.getByLabel("Project-relative file path",{exact:true}).fill("framework.json");await page.getByLabel("Stable resource key",{exact:true}).fill("framework");await page.getByRole("button",{name:"Preview registration",exact:true}).click();
     await page.route("**/api/v1/effects/commits",route=>route.fulfill({status:409,contentType:"application/json",body:JSON.stringify({code:"receipt-expired",message:"The preview expired. Prepare a new preview.",retryable:false})}),{times:1});
     await page.getByRole("dialog").getByRole("button",{name:"Confirm this exact write"}).click();const alert=page.getByRole("dialog").getByRole("alert");await alert.waitFor();assert.match(await alert.textContent(),/receipt-expired/);assert(await alert.evaluate(node=>node===document.activeElement));
     await page.getByRole("dialog").getByRole("button",{name:"Keep editing",exact:true}).click();assert.equal(await page.getByLabel("Project-relative file path",{exact:true}).inputValue(),"framework.json");
     await page.getByRole("button",{name:"Preview registration",exact:true}).click();await page.getByRole("dialog").waitFor();assert.equal(await page.getByRole("dialog").getByRole("alert").count(),0,"a new preview must not retain the previous confirmation error");
     await page.route("**/api/v1/resources?*",route=>route.fulfill({status:500,contentType:"application/json",body:JSON.stringify({code:"internal-error",message:"Synthetic refresh failure.",retryable:true})}),{times:1});
     await confirm();await page.locator("#error").waitFor({state:"visible"});assert.match(await page.locator("#error").textContent(),/write was saved.*Synthetic refresh failure/);assert(await page.locator("#error").evaluate(node=>node===document.activeElement));await page.getByRole("button",{name:"Refresh",exact:true}).click();await page.getByLabel("Output model").waitFor();
     await page.getByLabel("Output model").selectOption("oscal-catalog");await page.getByLabel("Output project-relative path").fill("converted.json");await page.getByRole("button",{name:"Prepare conversion"}).click();await confirm();
     await register("converted.json","oscal-catalog-artifact","converted");
     await navigate("Framework Scope");await page.getByLabel("Framework Catalog",{exact:true}).selectOption({label:"framework · framework.json"});await page.getByLabel("New decision manifest path within project").fill("scope.json");await page.getByRole("button",{name:"Preview initial scope manifest"}).click();await confirm();
     await navigate("Policies & Artifacts");await register("scope.json","applicability-manifest","scope");
     await navigate("Framework Scope");
     await page.getByLabel("Control to review").selectOption("framework-a");await page.getByLabel("Explicit decision").selectOption("applicable");await page.getByLabel("Decision reviewer key").fill("reviewer");await page.getByLabel("Decision reviewer name").fill("Synthetic reviewer <script>" );await page.getByLabel("Decision review time (RFC3339)").fill("2026-09-10T00:00:00Z");await page.getByLabel("Decision rationale").fill("Explicit synthetic scope review");
     await page.getByRole("button",{name:"Apply decision to unsaved manifest"}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();
     await page.getByRole("button",{name:"Analyze committed scope decisions"}).click();await confirm();
     await navigate("Mappings");
     await page.getByLabel("Framework Catalog",{exact:true}).selectOption({label:"framework · framework.json"});await page.getByLabel("Policy Catalog",{exact:true}).selectOption({label:"converted · converted.json"});await page.getByLabel("New decision manifest path within project").fill("mapping-manifest.json");await page.getByLabel("Mapping review scope").selectOption("control-only");
     for(const [label,value] of [["Stable collection key","mapping"],["Mapping collection title","Synthetic mapping"],["Document version","1"],["Review time (RFC3339, including timezone)","2026-09-10T00:00:00Z"],["Reviewer key","reviewer"],["Asserted reviewer name","Synthetic mapping reviewer"],["Intended use and limitations","Synthetic explicit review only"],["Stable relationship key","reviewed-none"],["Relationship rationale","Explicit absence of a positive relationship"]])await page.getByLabel(label,{exact:true}).fill(value);
     await page.getByLabel("Review matching rationale").selectOption("semantic");await page.getByRole("button",{name:"Load selected Catalog subjects"}).click();
     await page.getByLabel("Reviewed policy subject").selectOption({index:1});await page.getByLabel("Reviewed framework subject").selectOption({label:"Control: framework-a"});await page.getByLabel("Reviewed relationship",{exact:true}).selectOption("no-relationship");
     await page.getByRole("button",{name:"Preview initial mapping manifest"}).click();await confirm();
     await navigate("Policies & Artifacts");await register("mapping-manifest.json","mapping-collection","mapping");await navigate("Mappings");
     await page.getByLabel("Relationship to review").selectOption("reviewed-none");await page.getByLabel("Mapping review rationale").fill("Explicit updated rationale");await page.getByRole("button",{name:"Apply relationship to unsaved manifest"}).click();await page.getByRole("button",{name:"Validate decisions"}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();
     await page.getByLabel("New relationship key",{exact:true}).fill("reviewed-additional");await page.getByLabel("New relationship policy subject",{exact:true}).selectOption({index:1});await page.getByLabel("New relationship framework subject",{exact:true}).selectOption({label:"framework-b"});await page.getByLabel("New relationship type",{exact:true}).selectOption("equivalent-to");await page.getByLabel("New relationship reviewer",{exact:true}).selectOption("reviewer");await page.getByLabel("New relationship review time (RFC3339)",{exact:true}).fill("2026-09-10T00:00:00Z");await page.getByLabel("New relationship rationale",{exact:true}).fill("Explicit additional synthetic review");await page.getByRole("button",{name:"Add relationship to unsaved manifest",exact:true}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();
     await page.getByRole("button",{name:"Rebuild committed mapping collection"}).click();await confirm();
     await navigate("Policies & Artifacts");await register("mapping-collection.json","mapping-collection","built-mapping");
     await navigate("Framework Scope");await page.getByLabel("Reviewed mapping collection",{exact:true}).selectOption({label:"built-mapping · mapping-collection.json"});await page.getByRole("button",{name:"Link collection to unsaved scope",exact:true}).click();await page.getByRole("button",{name:"Preview decision changes"}).click();await confirm();await page.getByRole("button",{name:"Analyze committed scope decisions"}).click();await confirm();
     assert(await page.getByRole("cell",{name:"applicable-reviewed-no-relationship",exact:true}).count()>0);
     await navigate("Trace & Reports");await page.getByRole("button",{name:"Trace converted",exact:true}).click();await page.getByRole("heading",{name:"Provenance references",exact:true}).waitFor();
     await navigate("Trace & Reports");await page.getByLabel("Report",{exact:true}).selectOption("trace");await page.getByLabel("Report destination within project").fill("trace.html");await page.getByRole("button",{name:"Prepare export"}).click();await confirm();
     const download=page.waitForEvent("download");await page.getByRole("button",{name:"Download committed redacted report"}).click();assert.equal((await download).suggestedFilename(),"forge-redacted-report.html");
   }
   if(process.env.FORGE_TEST_SCREENSHOT)await page.screenshot({path:process.env.FORGE_TEST_SCREENSHOT,fullPage:true});
   await page.setViewportSize({width:640,height:900});
   assert(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth));
   assert.deepEqual(await context.cookies(),[]);
   assert.deepEqual(await page.evaluate(()=>[localStorage.length,sessionStorage.length]),[0,0]);
   await page.getByRole("button",{name:"Stop workspace",exact:true}).first().click();
   await page.getByRole("dialog").getByRole("button",{name:"Stop workspace",exact:true}).click();
   await page.getByText("Stopped",{exact:true}).waitFor();
   assert.deepEqual(violations,[]);assert.deepEqual(errors,[]);
   console.log(JSON.stringify({mode:readOnly?"read-only":"writable",documentedRequests:[...calls].sort(),nonLoopbackRequests:0,pageErrors:0}));
 } catch(error){
   console.error(error.message);console.error("Visible status:",await page.locator("#error").textContent());
   await page.screenshot({path:process.env.FORGE_TEST_SCREENSHOT||"/tmp/forge-workspace-browser-failure.png",fullPage:true});process.exitCode=1;
 } finally {await context.close();await browser.close();}
})().catch(error=>{console.error(error.message);process.exitCode=1;});
