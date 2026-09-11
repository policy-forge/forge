#!/usr/bin/env python3
"""Synthetic maintained-client conformance over the documented local API."""
import argparse
import base64
import json
from pathlib import Path
import tempfile
import uuid
from workspace_client import Workspace, WorkspaceError

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument("--forge",required=True)
args=parser.parse_args()
with tempfile.TemporaryDirectory(prefix="forge-client-") as directory:
    root=Path(directory)
    with Workspace(args.forge,root,read_only=False) as client:
        source=b"# Synthetic policy\n\n## Access\n\n- Operators must review supplied clauses.\n"
        prepared=client.request("POST","/api/v1/resources/upload",{"role":"policy-source","target_path":"policy.md","filename":"policy.md","content_base64":base64.b64encode(source).decode()},idempotency_key=str(uuid.uuid4()))
        preview=prepared["preview"]
        assert not (root/"policy.md").exists()
        key=str(uuid.uuid4())
        first=client.commit(preview,confirmed=True,idempotency_key=key)
        second=client.commit(preview,confirmed=True,idempotency_key=key)
        assert first==second and (root/"policy.md").read_bytes()==source
        def register(file,role,key):
            response=client.request("POST","/api/v1/resources/register",{"path":file,"role":role,"key":key},idempotency_key=str(uuid.uuid4()))
            client.commit(response["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        register("policy.md","policy-source","policy")
        resources=client.request("GET","/api/v1/resources")["page"]["items"]
        operation=client.wait(client.request("POST","/api/v1/conversions",{"source_resource_id":resources[0]["resource_id"],"output_kind":"oscal-catalog","target_path":"catalog.json"},idempotency_key=str(uuid.uuid4())))
        assert operation["state"]=="succeeded"
        client.commit(operation["result"]["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        register("catalog.json","oscal-catalog-artifact","catalog")
        resources=client.request("GET","/api/v1/resources")["page"]["items"]
        catalog=next(item for item in resources if item["key"]=="catalog")
        framework={"catalog":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":{"title":"Synthetic framework","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":"framework-a","title":"Synthetic control"}]}}
        upload=client.request("POST","/api/v1/resources/upload",{"role":"oscal-catalog-artifact","target_path":"framework.json","filename":"framework.json","content_base64":base64.b64encode(json.dumps(framework).encode()).decode()},idempotency_key=str(uuid.uuid4()))
        client.commit(upload["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        register("framework.json","oscal-catalog-artifact","framework")
        resources=client.request("GET","/api/v1/resources")["page"]["items"]
        framework=next(item for item in resources if item["key"]=="framework")
        subjects=client.request("GET","/api/v1/mapping/subjects?side=policy&resource_id="+catalog["resource_id"])["page"]["items"]
        subject=next(item for item in subjects if item["statement_count"]==0)
        mapping={"source_resource_id":catalog["resource_id"],"target_resource_id":framework["resource_id"],"target_path":"mapping.json","scope":"control-only","maps":[{"key":"reviewed-none","relationship":"no-relationship","sources":[{"type":"control","id_ref":subject["label"]}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}],"review":{"collection":{"key":"mapping","title":"Synthetic mappings","version":"1","last_modified":"2026-09-10T00:00:00Z"},"reviewers":[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Explicit synthetic review","reviewer_keys":["reviewer"],"reviewed_at":"2026-09-10T00:00:00Z"}}}
        initial_map=client.request("POST","/api/v1/mapping/initializations",mapping,idempotency_key=str(uuid.uuid4()))
        client.commit(initial_map["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        register("mapping.json","mapping-collection","mapping")
        map_draft=client.request("GET","/api/v1/mapping/draft")
        map_draft["manifest"]["mapping"]["maps"][0]["rationale"]="Updated explicit rationale"
        edit=client.request("PUT","/api/v1/mapping/draft",{"manifest":map_draft["manifest"],"observed_version":map_draft["version"]},idempotency_key=str(uuid.uuid4()))
        client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        built=client.wait(client.request("POST","/api/v1/mapping/builds",{},idempotency_key=str(uuid.uuid4())))
        assert built["state"]=="succeeded" and built["result"]["positive_relationship_count"]==0
        client.commit(built["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        register("mapping-collection.json","mapping-collection","built-mapping")
        initial=client.request("POST","/api/v1/applicability/initializations",{"framework_resource_id":framework["resource_id"],"target_path":"scope.json"},idempotency_key=str(uuid.uuid4()))
        client.commit(initial["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        register("scope.json","applicability-manifest","scope")
        controls=client.request("GET","/api/v1/applicability/controls")["page"]["items"]
        assert controls and all(item["decision_state"]=="under-review" for item in controls)
        draft=client.request("GET","/api/v1/applicability/draft")
        manifest=draft["manifest"]
        manifest["mapping_collections"]=["mapping-collection.json"]
        manifest["reviewers"]=[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}]
        manifest["decisions"]=[{"control_id":controls[0]["control_id"],"state":"applicable","reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}]
        assert client.request("POST","/api/v1/applicability/draft/validation",{"manifest":manifest})["state"]=="valid"
        edit=client.request("PUT","/api/v1/applicability/draft",{"manifest":manifest,"observed_version":draft["version"]},idempotency_key=str(uuid.uuid4()))
        client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        analysis=client.wait(client.request("POST","/api/v1/applicability/analyses",{},idempotency_key=str(uuid.uuid4())))
        assert analysis["state"]=="succeeded"
        client.commit(analysis["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        export=client.wait(client.request("POST","/api/v1/exports",{"report_kind":"trace","target_path":"report.html"},idempotency_key=str(uuid.uuid4())))
        client.commit(export["result"]["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
        downloaded=client.request("GET","/api/v1/exports/"+export["operation_id"]+"/download",raw=True)
        assert downloaded==(root/"report.html").read_bytes() and b"Synthetic reviewer" not in downloaded
    with Workspace(args.forge,root,read_only=True) as client:
        assert client.request("GET","/api/v1/project/summary")["resource_counts"]["total"]==6
        try:
            client.request("POST","/api/v1/resources/register",{"path":"report.html","role":"trace-report","key":"report"},idempotency_key=str(uuid.uuid4()))
        except WorkspaceError as error:
            assert error.payload["code"]=="read-only-session"
        else:
            raise AssertionError("Read-only session accepted a write")
print("Maintained headless client: upload, registration, conversion, mapping initialization/edit/build, scope decisions, analysis, trace export, idempotent commit, read-only and shutdown passed.")
