/** Exercise actual installed Chrome against one matching native metadata API2 2.2 or2.3 session. */
const assert=require('node:assert/strict');const fs=require('node:fs');const path=require('node:path');const {createHash}=require('node:crypto');
const [origin,mode,project,incoming,out,root]=process.argv.slice(2);
assert.match(origin,/^http:\/\/127\.0\.0\.1:[0-9]+$/);assert(['writable','read-only'].includes(mode));
assert(root && project && incoming && out);assert(!fs.existsSync(out));fs.mkdirSync(out);
const {chromium}=require(path.join(root,'ui/node_modules/playwright'));
/** Bind retained observations to bytes without retaining source content or capabilities. */
function sha(bytes){return createHash('sha256').update(bytes).digest('hex');}
/** Pin only files in the explicitly authored project, refusing symbolic fixture aliases. */
function files(directory=project,prefix=''){
  const result={};for(const name of fs.readdirSync(directory).sort()){
    const full=path.join(directory,name);const stat=fs.lstatSync(full);assert(!stat.isSymbolicLink(),'Fixture must not contain symlinks');
    const key=prefix+name;if(stat.isDirectory())Object.assign(result,files(full,key+'/'));else if(stat.isFile())result[key]={sha256:sha(fs.readFileSync(full)),bytes:stat.size};
  }return result;
}
const sourceJS=fs.readFileSync(path.join(root,'ui/workspace.js'));const sourceCSS=fs.readFileSync(path.join(root,'ui/workspace.css'));
const before=files();const indexBefore=fs.readFileSync(path.join(project,'forge.workspace.json'));const supplied=JSON.parse(fs.readFileSync(incoming));
const receipt={format:'forge.s6-native-browser-development/1',status:'error',mode,api_major:2,contract_version:null,phase:'launch',
 source_js_sha256:sha(sourceJS),source_css_sha256:sha(sourceCSS),
 script_sha256:sha(fs.readFileSync(__filename)),node_version:process.version,fixture_before:before,chosen_file_sha256:sha(fs.readFileSync(incoming)),
 focus_checks:[],reflow:[],requests:[],page_errors:0,non_loopback_requests:0,screenshots:[],
 limitations:['Actual native browser observations only if this campaign executes successfully.','Synthetic explicitly owned files; no source-content or multi-file restore.','Request routing observes/permits real loopback traffic only; no response or app-state substitution.','Page-context nonloopback observation/abort is not OS-wide network denial.','No AT/WCAG/human/Windows/Linux browser/platform acceptance or fullS6 closure.']};
(async()=>{
 let browser,page;
 try{
  browser=await chromium.launch({headless:true,executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',args:['--disable-background-networking']});receipt.browser_version=browser.version();
  receipt.playwright_version=JSON.parse(fs.readFileSync(path.join(root,'ui/node_modules/playwright/package.json'))).version;
  const context=await browser.newContext({viewport:{width:1440,height:1000},serviceWorkers:'block',acceptDownloads:true});page=await context.newPage();page.setDefaultTimeout(20000);
  const requests=new Set();page.on('pageerror',()=>receipt.page_errors++);
  await context.route('**/*',async route=>{
   const request=route.request();const target=new URL(request.url());
   if(target.origin!==origin){receipt.non_loopback_requests++;await route.abort();return;}
   if(target.pathname.startsWith('/api/')){
    assert(target.pathname.startsWith('/api/v2/'));requests.add(request.method()+' '+target.pathname.replace(/(?:res|op|prev)_[0-9a-z]+/g,'{id}'));receipt.requests=[...requests].sort();
   }await route.continue();
  });
  /** Wait for observed connected focus without assigning the expected element to satisfy the predicate. */
  async function focused(locator,label){
   const deadline=Date.now()+20000;while(!await locator.evaluate(element=>element===document.activeElement)){assert(Date.now()<deadline,'Expected actual connected focus');await page.waitForTimeout(25);}receipt.focus_checks.push(label);
  }
  /** Save actual viewport pixels and retain their phase, leaving visual acceptance to ROOT/human review. */
  async function screenshot(name){await page.screenshot({path:path.join(out,name),fullPage:false});receipt.screenshots.push(name);}
  /** Record both global page widths before asserting, allow internal tables to scroll, and restore the actual viewport. */
  async function reflow(label){
   const initial=page.viewportSize();const rows=[];
   for(const width of [640,320]){await page.setViewportSize({width,height:1000});await page.waitForTimeout(50);const dimensions=await page.evaluate(()=>({viewport:window.innerWidth,page:document.documentElement.scrollWidth}));rows.push({phase:label,...dimensions});await screenshot(label+'-'+width+'.png');}
   await page.setViewportSize(initial);receipt.reflow.push(...rows);assert(rows.every(row=>row.page<=row.viewport),'Global page overflow observed');
  }
  /** Prepare through the native initiating control and verify focused exact-receipt review. */
  async function prepared(button){await button.focus();await button.press('Enter');await page.locator('#preview-dialog[open]').waitFor();await focused(page.locator('#preview-title'),'exact-receipt-heading');}
  /** Dismiss through actual native Enter or Escape and verify return to the still-connected initiating control. */
  async function dismissed(button,escape=false){if(escape)await page.locator('#preview-title').press('Escape');else{const keep=page.getByRole('button',{name:'Keep editing',exact:true});await keep.focus();await keep.press('Enter');}await page.locator('#preview-dialog[open]').waitFor({state:'hidden'});await focused(button,escape?'Escape-return-target':'Keep-editing-return-target');}
  /** Confirm only the exact visible receipt and await real saved publication after native modal closure. */
  async function confirmed(target){const button=page.getByRole('button',{name:'Confirm this exact write',exact:true});await button.focus();await button.press('Enter');await page.waitForFunction(value=>document.getElementById('status').textContent==='Saved '+value+'.',target);await page.locator('#preview-dialog[open]').waitFor({state:'hidden'});await focused(page.locator('#view-title'),'verified-saved-heading');}
  const asset=page.waitForResponse(response=>/\/assets\/[a-f0-9]{64}\.js$/.test(new URL(response.url()).pathname));const style=page.waitForResponse(response=>/\/assets\/[a-f0-9]{64}\.css$/.test(new URL(response.url()).pathname));
  await page.goto(origin);assert.deepEqual(await(await asset).body(),sourceJS);assert.deepEqual(await(await style).body(),sourceCSS);
  assert.equal(await page.locator('meta[name="forge-api-major"]').getAttribute('content'),'2');const negotiated=await page.locator('meta[name="forge-api-contract-version"]').getAttribute('content');assert(['2.2.0','2.3.0'].includes(negotiated));receipt.contract_version=negotiated;
  const passphrase=page.getByLabel('Workspace passphrase');await focused(passphrase,'initial-passphrase');await passphrase.fill('synthetic S6 browser bundle passphrase 062');await passphrase.press('Enter');
  await page.waitForFunction(()=>document.getElementById('status').textContent==='Project state loaded.');
  const navigation=page.getByRole('button',{name:'Trace & Reports',exact:true});await navigation.focus();await navigation.press('Enter');await page.waitForFunction(()=>document.getElementById('view-title').textContent==='Trace & Reports');await focused(page.locator('#view-title'),'Trace-heading');
  const panel=page.locator('[data-bundle-effects]');await panel.waitFor();assert.match(await panel.textContent(),/Export metadata or replace the index/);
  const preview=page.getByRole('button',{name:'Preview metadata',exact:true});await preview.focus();await preview.press('Enter');await page.locator('[data-bundle-preview-status]').filter({hasText:'2 registered resources'}).waitFor();
  const globalStatus=await page.locator('#status').textContent();assert.deepEqual(files(),before);receipt.phase='actual-query';await screenshot('metadata-query.png');assert.equal(await page.locator('#view script').count(),0);
  const localAck=page.getByLabel('I understand that labels, resource keys, paths and hashes can reveal project information.',{exact:true});await localAck.focus();await localAck.press('Space');
  const localDownload=page.getByRole('button',{name:'Download metadata bundle',exact:true});await localDownload.focus();const localPending=page.waitForEvent('download');await localDownload.press('Enter');const local=await localPending;assert.equal(local.suggestedFilename(),'forge-workspace-index-and-hashes.json');await local.saveAs(path.join(out,'query-metadata.json'));
  const localBundle=JSON.parse(fs.readFileSync(path.join(out,'query-metadata.json')));assert.equal(localBundle.index.resources.length,2);assert.equal(localBundle.index.schema_version,'forge.workspace/1');assert.deepEqual(files(),before);assert.equal(await page.locator('#status').textContent(),globalStatus);await focused(localDownload,'local-query-download-control');receipt.local_query_download_no_project_write=true;
  const exportTarget=page.getByLabel('Metadata export project-relative target');const sensitive=page.getByLabel('I acknowledge that exported labels, keys, paths and hashes are sensitive metadata.',{exact:true});const exportButton=page.getByRole('button',{name:'Prepare metadata export',exact:true});
  const selectedFile=page.getByLabel('Choose a bundle for index replacement');const schema=page.getByLabel('Replacement target index schema');const ack=page.getByLabel('I acknowledge replacing the index label and all registrations; source files will not be deleted.',{exact:true});const importButton=page.getByRole('button',{name:'Prepare index replacement',exact:true});
  await exportTarget.fill('metadata-export.json');assert.equal(await sensitive.isChecked(),false);assert.equal(await exportButton.getAttribute('aria-disabled'),'true');
  await exportTarget.focus();await exportTarget.press('Tab');await focused(sensitive,'native-Tab-sensitivity-checkbox');await sensitive.press('Space');assert.equal(await sensitive.isChecked(),true);
  if(mode==='writable'){
   assert.equal(await exportButton.getAttribute('aria-disabled'),'false');receipt.phase='export-dismissal';await prepared(exportButton);await dismissed(exportButton);assert.deepEqual(files(),before);
   await prepared(exportButton);await dismissed(exportButton,true);assert.deepEqual(files(),before);
   receipt.phase='export-confirmation';await prepared(exportButton);await screenshot('export-exact-receipt.png');await confirmed('metadata-export.json');
   const exported=fs.readFileSync(path.join(project,'metadata-export.json'));assert.equal(fs.readFileSync(path.join(project,'forge.workspace.json')).equals(indexBefore),true);receipt.export_artifact_sha256=sha(exported);
   const downloadButton=page.getByRole('button',{name:'Download committed metadata bundle',exact:true});await downloadButton.focus();const pendingDownload=page.waitForEvent('download');await downloadButton.press('Enter');const downloaded=await pendingDownload;assert.equal(downloaded.suggestedFilename(),'forge-workspace-index-and-hashes.json');await downloaded.saveAs(path.join(out,'committed-metadata.json'));assert.deepEqual(fs.readFileSync(path.join(out,'committed-metadata.json')),exported);await focused(downloadButton,'committed-download-control');
   receipt.committed_download_filename=downloaded.suggestedFilename();receipt.committed_download_exact_bytes=true;
  }else{
   assert.equal(await exportButton.getAttribute('aria-disabled'),'true');await exportButton.focus();await exportButton.press('Enter');assert.deepEqual(files(),before);
  }
  receipt.phase='chosen-replacement';await selectedFile.setInputFiles(incoming);await schema.selectOption('2');assert.equal(await ack.isChecked(),false);
  const chosen=await selectedFile.evaluate(input=>({name:input.files[0].name,size:input.files[0].size}));assert.equal(chosen.name,path.basename(incoming));assert.equal(chosen.size,fs.statSync(incoming).size);receipt.chosen_file_properties=chosen;
  await ack.focus();await ack.press('Space');assert.equal(await ack.isChecked(),true);assert.equal(await schema.inputValue(),'2');await reflow('chosen-file');
  if(mode==='writable'){
   const unconfirmed=files();await prepared(importButton);const content=page.locator('#preview-content');assert.match(await content.textContent(),/Complete project index replacement/);assert.match(await content.textContent(),/forge\.workspace\/1/);assert.match(await content.textContent(),/forge\.workspace\/2/);assert.match(await content.textContent(),/Removed registration keys: policy/);
   const oldMembership=page.getByRole('region',{name:'Previous index complete membership',exact:true});const newMembership=page.getByRole('region',{name:'Proposed index complete membership',exact:true});assert.equal(await oldMembership.locator('tbody tr').count(),2);assert.equal(await newMembership.locator('tbody tr').count(),2);assert.match(await newMembership.textContent(),/incoming/);
   await screenshot('import-complete-replacement.png');await reflow('replacement-dialog');await focused(page.locator('#preview-title'),'replacement-heading-after-reflow');await dismissed(importButton);assert.deepEqual(files(),unconfirmed);
   await prepared(importButton);await dismissed(importButton,true);assert.deepEqual(files(),unconfirmed);
   receipt.phase='import-confirmation';await prepared(importButton);await confirmed('forge.workspace.json');
   const committed=JSON.parse(fs.readFileSync(path.join(project,'forge.workspace.json')));assert.equal(committed.schema_version,'forge.workspace/2');assert.equal(committed.label,supplied.index.label);assert.deepEqual(committed.resources,supplied.index.resources);
   const after=files();for(const [name,pin] of Object.entries(before))if(name!=='forge.workspace.json')assert.deepEqual(after[name],pin);assert.deepEqual(after['metadata-export.json'],unconfirmed['metadata-export.json']);receipt.index_only_replacement=true;receipt.full_previous_resources=2;receipt.full_proposed_resources=2;receipt.source_bytes_unchanged=true;
   await screenshot('confirmed-index-only-replacement.png');
  }else{
   assert.equal(await importButton.getAttribute('aria-disabled'),'true');await importButton.focus();await importButton.press('Enter');assert.equal(await page.locator('#preview-dialog[open]').count(),0);assert.deepEqual(files(),before);assert.equal(await page.locator('#status').textContent(),globalStatus);receipt.readonly_project_unchanged=true;
   assert(!receipt.requests.some(value=>/^POST \/api\/v2\/(project\/bundle-(?:exports|imports)|effects\/commits)/.test(value)));
   await screenshot('readonly-preparation-guards.png');
  }
  receipt.phase='native-shutdown';const stop=page.locator('#stop');await stop.focus();await stop.press('Enter');const confirmStop=page.locator('#confirm-stop');await confirmStop.focus();await confirmStop.press('Enter');await page.waitForFunction(()=>document.getElementById('status').textContent.startsWith('Workspace stopped'));await focused(page.locator('#main'),'stopped-main');
  receipt.shutdown_UI_observed=true;receipt.fixture_after=files();assert.deepEqual(fs.readFileSync(path.join(root,'ui/workspace.js')),sourceJS);assert.deepEqual(fs.readFileSync(path.join(root,'ui/workspace.css')),sourceCSS);receipt.source_assets_unchanged=true;assert.equal(receipt.page_errors,0);assert.equal(receipt.non_loopback_requests,0);receipt.status='passed';receipt.phase='complete';
 }catch(error){receipt.error_class=error.constructor.name;process.exitCode=1;if(page)try{await page.screenshot({path:path.join(out,'failure.png'),fullPage:false});receipt.screenshots.push('failure.png');}catch{}}
 finally{if(browser)await browser.close();fs.writeFileSync(path.join(out,'receipt.json'),JSON.stringify(receipt,null,2)+'\n');process.stdout.write(JSON.stringify({receipt:path.join(out,'receipt.json'),status:receipt.status,phase:receipt.phase})+'\n');}
})();
