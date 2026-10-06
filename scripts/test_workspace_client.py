#!/usr/bin/env python3
"""Synthetic maintained-client conformance over the documented local API."""
import argparse
import base64
import json
import hashlib
from pathlib import Path
import tempfile
import uuid
import sys

sys.dont_write_bytecode = True
from verify_workspace import MAX_CLIENT_REQUESTS, atomic_receipt, contract_routes
from workspace_client import Workspace, WorkspaceError

# Limit actual request attempts across the two synchronous sessions; retain only
# allowlisted contract identifiers and three integer outcome categories.
MAX_RECORDED_REQUESTS = MAX_CLIENT_REQUESTS


class RecordingWorkspace(Workspace):
    """Record documented operation identifiers and bounded outcomes only."""

    # Shared only within one synchronous run; run() resets it before both sessions.
    request_count = 0
    accounting_failed = False

    def record(self, operation, outcome):
        """Increment one allowlisted outcome without retaining request or response data."""
        if outcome not in ("succeeded", "rejected", "transport_failed"):
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client used an unsupported outcome category")
        try:
            self.outcomes.setdefault(operation, {
                "succeeded": 0, "rejected": 0, "transport_failed": 0,
            })[outcome] += 1
        except Exception:
            RecordingWorkspace.accounting_failed = True
            raise

    def request(self, method, path, body=None, **kwargs):
        """Forward one documented request and retain only its operation outcome."""
        try:
            operation = next((name for verb, pattern, name in self.routes
                              if verb == method and pattern.fullmatch(path.split("?", 1)[0])), None)
        except Exception:
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client could not classify its request") from None
        if operation is None:
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client used an undocumented operation")
        if (type(RecordingWorkspace.request_count) is not int or
                not 0 <= RecordingWorkspace.request_count < MAX_RECORDED_REQUESTS):
            RecordingWorkspace.accounting_failed = True
            raise RuntimeError("The client exceeded its request accounting bound")
        RecordingWorkspace.request_count += 1
        try:
            result = super().request(method, path, body, **kwargs)
        except WorkspaceError:
            self.record(operation, "rejected")
            raise
        except OSError:
            self.record(operation, "transport_failed")
            raise
        except Exception:
            # close() may swallow RuntimeError/ValueError after the server has
            # already honoured shutdown; retain a failure latch without its text.
            RecordingWorkspace.accounting_failed = True
            raise
        self.record(operation, "succeeded")
        return result


    @classmethod
    def ensure_accounting(cls):
        """Reject sticky failures and require every attempted request to have one closed outcome."""
        if cls.accounting_failed:
            raise RuntimeError("The client could not account for every request")
        try:
            if type(cls.request_count) is not int or not 0 <= cls.request_count <= MAX_RECORDED_REQUESTS:
                raise ValueError("Invalid attempt count")
            declared = {name for _, _, name in cls.routes}
            total = 0
            for operation, counts in cls.outcomes.items():
                if operation not in declared or set(counts) != {"succeeded", "rejected", "transport_failed"}:
                    raise ValueError("Invalid outcome inventory")
                if any(type(count) is not int or count < 0 for count in counts.values()):
                    raise ValueError("Invalid outcome count")
                total += sum(counts.values())
            if total != cls.request_count:
                raise ValueError("Unreconciled attempt count")
        except Exception:
            cls.accounting_failed = True
            raise RuntimeError("The client could not account for every request") from None



def run(forge):
    """Run all existing workflows and publish only complete assertion-group accounting."""
    if not __debug__:
        raise RuntimeError("Conformance assertions require Python without optimization")
    checks = []
    RecordingWorkspace.routes = contract_routes(Path(__file__).resolve().parents[1])
    RecordingWorkspace.outcomes = {}
    RecordingWorkspace.request_count = 0
    RecordingWorkspace.accounting_failed = False

    with tempfile.TemporaryDirectory(prefix="forge-client-") as directory:
        root=Path(directory)
        with RecordingWorkspace(forge,root,read_only=False) as client:
            source=b"# Synthetic policy\n\n## Access\n\n- Operators must review supplied clauses.\n"
            prepared=client.request("POST","/api/v1/resources/upload",{"role":"policy-source","target_path":"policy.md","filename":"policy.md","content_base64":base64.b64encode(source).decode()},idempotency_key=str(uuid.uuid4()))
            preview=prepared["preview"]
            assert not (root/"policy.md").exists()
            checks.append("upload_requires_confirmation")
            key=str(uuid.uuid4())
            first=client.commit(preview,confirmed=True,idempotency_key=key)
            second=client.commit(preview,confirmed=True,idempotency_key=key)
            assert first==second and (root/"policy.md").read_bytes()==source
            checks.append("idempotent_commit_replays_exact_bytes")
            def register(file,role,key):
                """Prepare and explicitly commit one documented resource registration."""
                response=client.request("POST","/api/v1/resources/register",{"path":file,"role":role,"key":key},idempotency_key=str(uuid.uuid4()))
                client.commit(response["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("policy.md","policy-source","policy")
            resources=client.request("GET","/api/v1/resources")["page"]["items"]
            operation=client.wait(client.request("POST","/api/v1/conversions",{"source_resource_id":resources[0]["resource_id"],"output_kind":"oscal-catalog","target_path":"catalog.json"},idempotency_key=str(uuid.uuid4())))
            assert operation["state"]=="succeeded"
            checks.append("conversion_succeeds")
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
            checks.append("explicit_no_relationship_is_preserved")
            client.commit(built["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("mapping-collection.json","mapping-collection","built-mapping")
            initial=client.request("POST","/api/v1/applicability/initializations",{"framework_resource_id":framework["resource_id"],"target_path":"scope.json"},idempotency_key=str(uuid.uuid4()))
            client.commit(initial["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            register("scope.json","applicability-manifest","scope")
            controls=client.request("GET","/api/v1/applicability/controls")["page"]["items"]
            assert controls and all(item["decision_state"]=="under-review" for item in controls)
            checks.append("omitted_scope_decisions_remain_under_review")
            draft=client.request("GET","/api/v1/applicability/draft")
            manifest=draft["manifest"]
            manifest["mapping_collections"]=["mapping-collection.json"]
            manifest["reviewers"]=[{"key":"reviewer","type":"person","name":"Synthetic reviewer"}]
            manifest["decisions"]=[{"control_id":controls[0]["control_id"],"state":"applicable","reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit synthetic review"}]
            assert client.request("POST","/api/v1/applicability/draft/validation",{"manifest":manifest})["state"]=="valid"
            checks.append("explicit_scope_decision_validates")
            edit=client.request("PUT","/api/v1/applicability/draft",{"manifest":manifest,"observed_version":draft["version"]},idempotency_key=str(uuid.uuid4()))
            client.commit(edit["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            analysis=client.wait(client.request("POST","/api/v1/applicability/analyses",{},idempotency_key=str(uuid.uuid4())))
            assert analysis["state"]=="succeeded"
            checks.append("committed_scope_analysis_succeeds")
            client.commit(analysis["result"]["report_preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            export=client.wait(client.request("POST","/api/v1/exports",{"report_kind":"trace","target_path":"report.html"},idempotency_key=str(uuid.uuid4())))
            client.commit(export["result"]["preview"],confirmed=True,idempotency_key=str(uuid.uuid4()))
            downloaded=client.request("GET","/api/v1/exports/"+export["operation_id"]+"/download",raw=True)
            assert downloaded==(root/"report.html").read_bytes() and b"Synthetic reviewer" not in downloaded
            checks.append("export_matches_committed_redacted_bytes")
        writer=client
        with RecordingWorkspace(forge,root,read_only=True) as client:
            assert client.request("GET","/api/v1/project/summary")["resource_counts"]["total"]==6
            checks.append("read_only_summary_reconciles_resources")
            before_bundle_queries={path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}
            bundle_preview=client.bundle_preview()
            bundle=bundle_preview["bundle"]
            assert bundle_preview["source_index_present"] is True
            assert bundle_preview["source_content_included"] is False
            assert bundle["schema_version"]=="forge.workspace-index-bundle/1"
            assert bundle["content_profile"]=="index-and-hashes"
            assert len(bundle["pins"])==len(bundle["index"]["resources"])==6
            checks.append("metadata_bundle_preview_preserves_complete_denominator")
            normalized_index={"schema_version":bundle["index"]["schema_version"],"label":bundle["index"]["label"],"resources":[{"key":item["key"],"role":item["role"],"path":item["path"]} for item in bundle["index"]["resources"]]}
            normalized_bytes=(json.dumps(normalized_index,ensure_ascii=False,indent=2)+"\n").encode()
            assert hashlib.sha256(normalized_bytes).hexdigest()==bundle["index_sha256"]
            for registration,pin in zip(bundle["index"]["resources"],bundle["pins"]):
                resource_bytes=(root/registration["path"]).read_bytes()
                assert pin["key"]==registration["key"]
                assert pin["sha256"]==hashlib.sha256(resource_bytes).hexdigest()
                assert pin["size_bytes"]==len(resource_bytes)
            checks.append("metadata_bundle_fingerprints_match_exact_bytes")
            verified=client.verify_bundle(bundle)
            assert verified["scope"]=="registered-fingerprints-only" and verified["state"]=="matched"
            assert verified["expected_resources"]==verified["matched_resources"]==6
            assert verified["unregistered_resources"]==verified["mismatched_resources"]==0
            assert verified["current_resources"]==6 and verified["current_only_resources"]==0
            assert verified["expected_index_matches_current"] is True
            assert [item["key"] for item in verified["items"]]==[item["key"] for item in bundle["index"]["resources"]]
            assert verified["source_content_included"] is False
            checks.append("registered_bundle_comparison_reconciles_expected_current")
            assert {path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}==before_bundle_queries
            checks.append("metadata_bundle_queries_preserve_workspace_files")
            try:
                client.request("POST","/api/v1/resources/register",{"path":"report.html","role":"trace-report","key":"report"},idempotency_key=str(uuid.uuid4()))
            except WorkspaceError as error:
                assert error.payload["code"]=="read-only-session"
                checks.append("read_only_mutation_is_rejected")
            else:
                raise AssertionError("Read-only session accepted a write")
        reader=client
        # Workspace.close() swallows shutdown failures, so prove each session really
        # ended: a clean exit code means the shutdown route was honoured rather than
        # the close() timeout killing the process, and nothing may still answer.
        for session in (writer,reader):
            assert session.process.poll()==0, f"Workspace did not shut down cleanly: exit {session.process.poll()}"
            try:
                session.request("GET","/api/v1/project/summary")
            except OSError:
                checks.append("clean_shutdown_" + ("writable" if session is writer else "read_only"))
                continue
            except WorkspaceError:
                raise AssertionError("The workspace answered after close()") from None
            raise AssertionError("The workspace answered after close()")

    RecordingWorkspace.ensure_accounting()
    observed = sorted(RecordingWorkspace.outcomes)
    declared = sorted(name for _, _, name in RecordingWorkspace.routes)
    return {
        "schema_version": "forge.workspace-client-verification/2",
        "status": "passed",
        "checks": checks,
        "check_count": len(checks),
        "declared_operation_count": len(declared),
        "observed_operations": observed,
        "unobserved_operations": sorted(set(declared) - set(observed)),
        "operation_outcomes": RecordingWorkspace.outcomes,
    }


def main():
    """Run conformance and atomically retain an optional redacted receipt without replacement."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge", required=True)
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    # Existing destinations, including dangling symlinks, are preserved before
    # executing the workflow. Atomic hard-link publication also closes the race.
    if args.receipt and (args.receipt.exists() or args.receipt.is_symlink()):
        print("Maintained headless client receipt destination already exists.", file=sys.stderr)
        return 1
    try:
        receipt = run(args.forge)
    except Exception:
        if args.receipt:
            try:
                atomic_receipt(args.receipt, {
                    "schema_version": "forge.workspace-client-verification/2",
                    "status": "failed", "checks": [], "check_count": 0,
                    "failure": "client-conformance-failed",
                })
            except Exception:
                # Publication failures never expose native paths or exception
                # payloads. Exit status remains failed without a success receipt.
                pass
        print("Maintained headless client conformance failed.", file=sys.stderr)
        return 1
    if args.receipt:
        try:
            atomic_receipt(args.receipt, receipt)
        except Exception:
            print("Maintained headless client receipt publication failed.", file=sys.stderr)
            return 1
    print(f"Maintained headless client conformance passed ({receipt['check_count']} explicit check groups).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
