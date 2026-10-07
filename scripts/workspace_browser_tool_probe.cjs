"use strict";
/** Report only the exact approved UI-local dependency graph; never launch or download a browser. */
// Threat model: a local, read-only run with no concurrent writer to ui/node_modules or docs/. Pins below are deliberate
// and must change in lockstep with docs/development-tools/hosted-chrome.json; any mismatch, including the Node pin vs
// runtime.node in that manifest, is a hard failure by design.
const fs=require("node:fs"),path=require("node:path"),crypto=require("node:crypto"),Module=require("node:module");
const root=path.resolve(__dirname,"..");
/** Capture stable bounded regular descriptors; optionally retain only bounded JSON bytes. */
function readBounded(filename,maximum,retain=false) {
 const initial=fs.lstatSync(filename);if(!initial.isFile()||initial.isSymbolicLink()||initial.size>maximum)throw Error("invalid-file");
 const flags=fs.constants.O_RDONLY|(fs.constants.O_NOFOLLOW||0)|(fs.constants.O_NONBLOCK||0);
 const descriptor=fs.openSync(filename,flags);let count=0;const blocks=[];const digest=crypto.createHash("sha256");
 /** Compare descriptor/path metadata without treating a later unchanged hash as startup-byte attestation. */
 const same=(a,b)=>a.dev===b.dev&&a.ino===b.ino&&a.size===b.size&&a.mtimeMs===b.mtimeMs&&a.ctimeMs===b.ctimeMs;
 try {
  const before=fs.fstatSync(descriptor);if(!before.isFile()||!same(initial,before))throw Error("changed-file");
  const buffer=Buffer.alloc(65536);
  while(true){const read=fs.readSync(descriptor,buffer,0,buffer.length,null);if(!read)break;
   count+=read;if(count>maximum)throw Error("file-bound");digest.update(buffer.subarray(0,read));if(retain)blocks.push(Buffer.from(buffer.subarray(0,read)));}
  if(count!==before.size||!same(before,fs.fstatSync(descriptor))||!same(before,fs.lstatSync(filename)))throw Error("changed-file");
  return {pin:{bytes:count,sha256:digest.digest("hex")},raw:retain?Buffer.concat(blocks):null};
 }finally{fs.closeSync(descriptor);}
}
/** Hash package bytes in bounded memory without exposing their contents. */
function filePin(filename,maximum=16777216) {return readBounded(filename,maximum).pin;}
/** Stream a bounded immediate directory inventory before sorting or global graph rejection. */
function directoryNames(directory,maximum=64) {
 const names=[];const listing=fs.opendirSync(directory);
 try{let entry;while((entry=listing.readSync())!==null){if(names.length>=maximum)throw Error("directory-bound");names.push(entry.name);}}
 finally{listing.closeSync();}
 return names;
}
/** Retain a complete bounded package tree hash with all regular files and no links or nested dependencies. */
function packagePin(name,entry) {
 const directory=path.join(root,"ui/node_modules",name);if(fs.realpathSync(directory)!==directory)throw Error("nonlocal-package");
 const rows=[];let total=0,entries=0;
 /** Walk the approved package recursively; archive integrity and audit approval remain separate. */
 function visit(relative,depth=0) {
  if(depth>32)throw Error("package-depth");
  const absolute=path.join(directory,relative);const observed=fs.lstatSync(absolute);
  if(observed.isDirectory()){
   const listing=fs.opendirSync(absolute);
   try{let item;while((item=listing.readSync())!==null){if(++entries>10000)throw Error("package-entry-bound");visit(path.join(relative,item.name),depth+1);}}
   finally{listing.closeSync();}
   return;
  }
  const pin=filePin(absolute);total+=pin.bytes;if(rows.length>=10000||total>67108864)throw Error("package-bound");
  rows.push({path:relative.split(path.sep).join("/"),...pin});
 }
 visit("");rows.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);
 const manifest=JSON.parse(readBounded(path.join(directory,"package.json"),1048576,true).raw.toString("utf8"));
 // Security pin: reviewed on every dependency upgrade together with hosted-chrome.json.
 if(manifest.name!==name||manifest.version!=="1.62.1")throw Error("package-version");
 if(name==="playwright"&&(JSON.stringify(manifest.dependencies)!==JSON.stringify({"playwright-core":"1.62.1"})||manifest.optionalDependencies?.fsevents!=="2.3.2"))throw Error("package-graph");
 if(name==="playwright-core"&&manifest.dependencies&&Object.keys(manifest.dependencies).length)throw Error("package-graph");
 if(fs.realpathSync(entry)!==path.join(directory,"index.js"))throw Error("nonlocal-entry");
 return {name,version:manifest.version,relative_path:"ui/node_modules/"+name,manifest:filePin(path.join(directory,"package.json")),entry:filePin(entry),files:rows.length,bytes:total,tree_sha256:crypto.createHash("sha256").update(JSON.stringify(rows)).digest("hex")};
}
/** Refuse global resolution and optional/native packages before emitting the closed tool observation. */
function main() {
 // Exact runtime pin: any Node change, including patch updates, invalidates the attestation until deliberately updated.
 if(process.version!=="v24.19.0"||process.env.NODE_PATH||process.env.NODE_OPTIONS)throw Error("runtime-scope");
 const directory=path.join(root,"ui/node_modules");const names=directoryNames(directory).filter(name=>!name.startsWith(".")).sort();
 if(JSON.stringify(names)!==JSON.stringify(["playwright","playwright-core"]))throw Error("unapproved-package");
 const requireUi=Module.createRequire(path.join(root,"ui/tests/workspace.cjs"));
 const playwright=requireUi.resolve("playwright");const requirePlaywright=Module.createRequire(playwright);const core=requirePlaywright.resolve("playwright-core");
 const packages=[packagePin("playwright",playwright),packagePin("playwright-core",core)];
 const manifest=JSON.parse(readBounded(path.join(root,"docs/development-tools/hosted-chrome.json"),1048576,true).raw.toString("utf8"));
 if(typeof manifest.runtime?.node!=="string"||process.version!=="v"+manifest.runtime.node)throw Error("runtime-manifest");
 const expected=manifest.qualified_installed_package_trees.packages;
 if(expected.length!==2)throw Error("tree-scope");
 for(let index=0;index<2;index++){for(const field of ["name","files","bytes","tree_sha256"]){if(packages[index][field]!==expected[index][field])throw Error("archive-tree-mismatch");}
  for(const field of ["manifest","entry"]){if(JSON.stringify(packages[index][field])!==JSON.stringify(expected[index][field]))throw Error("archive-entry-mismatch");}}
 if(fs.existsSync(path.join(directory,"fsevents")))throw Error("optional-installed");
 process.stdout.write(JSON.stringify({schema_version:"forge.hosted-chrome-tools/1",node:manifest.runtime.node,packages,optional_omitted:true,installed_lock:filePin(path.join(directory,".package-lock.json"),1048576)})+"\n");
}
try{main();}catch(_error){process.stderr.write("Approved local browser tooling is unverified.\n");process.exitCode=1;}
