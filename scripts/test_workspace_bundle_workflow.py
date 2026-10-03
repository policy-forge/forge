#!/usr/bin/env python3
"""Real-loopback API2 2.2/2.3 metadata companion; no source-content or multifile restoration."""
import argparse
import hashlib
import http.client
import json
from pathlib import Path
import tempfile
import time
import uuid
from workspace_client import Workspace, WorkspaceError


def require(condition, message):
    """Fail with a static redacted explanation, never a wire capability, payload or filesystem path."""
    if not condition:
        raise RuntimeError(message)


def require_metadata_version(client):
    """Return the observed supported metadata version only for an explicitly selected API2 session."""
    version=client._contract_version
    require(client._api_major==2 and version in ("2.2.0", "2.3.0"),"Metadata companion requires selected API2 2.2.0 or 2.3.0")
    return version


def confirm(client, preview):
    """Confirm an already reviewed exact preview with its own original receipt and one private key."""
    result=client.wait(client.commit(preview,confirmed=True,idempotency_key=str(uuid.uuid4())))
    require(result["state"]=="succeeded","Exact confirmation did not succeed")
    require(result["result"]["write_committed"] is True,"Confirmation did not establish a write")
    require(result["result"]["committed_sha256"]==preview["exact_bytes_sha256"],"Committed hash differs from exact preview")
    return result


def shutdown(client):
    """Observe the selected-major shutdown response and normal process exit before context cleanup."""
    reply=client.request("POST",client.api_path("/session/shutdown"),{})
    require(reply=={"state":"shutting-down"},"Shutdown response differs from the closed contract")
    require(client.process.wait(timeout=5)==0,"Shutdown did not exit normally")



def project_bytes(project):
    """Retain every real fixture file privately so rejected preparations cannot hide source/index changes."""
    return {str(path.relative_to(project)):path.read_bytes() for path in project.rglob("*") if path.is_file()}


def raw_import(client, body, *, chunked=False, declared_length=None):
    """Send actual bounded loopback bytes, or an oversized declared header, without mocked observations."""
    require_metadata_version(client)
    require(type(body) is bytes,"Raw boundary driver requires original bytes")
    delay=client._next_request_at-time.monotonic()
    if delay>0:
        time.sleep(delay)
    client._next_request_at=time.monotonic()+0.06
    connection=http.client.HTTPConnection("127.0.0.1",client._port,timeout=30)
    headers={"Authorization":"Bearer "+client._capability,"Accept":"application/json","Content-Type":"application/json","Idempotency-Key":str(uuid.uuid4())}
    try:
        if declared_length is not None:
            # This control deliberately sends no body: it tests header admission, not aggregate-byte collection.
            connection.putrequest("POST",client.api_path("/project/bundle-imports"))
            for name,value in {**headers,"Content-Length":str(declared_length)}.items():
                connection.putheader(name,value)
            connection.endheaders()
        else:
            if chunked:
                headers["Transfer-Encoding"]="chunked"
            try:
                connection.request("POST",client.api_path("/project/bundle-imports"),body=body,headers=headers,encode_chunked=chunked)
            except BrokenPipeError:
                # A bound rejection may close after receiving the excessive chunk; only an actual typed response can pass.
                pass
        response=connection.getresponse();raw=response.read(1024*1024+1)
        require(len(raw)<=1024*1024,"Boundary response exceeds its bound")
        require(response.getheader("Content-Type","").split(";",1)[0].strip().lower()=="application/json","Boundary response is not typed JSON")
        return response.status,json.loads(raw)
    finally:
        connection.close()


def raw_wrapper(bundle_bytes, target=1):
    """Construct the accepted80-byte numeric envelope around exact chosen bytes without decoding them."""
    return b'{"bundle":'+bundle_bytes+(',"target_index_schema_version":'+str(target)+',"acknowledge_index_replacement":true}').encode("ascii")


def raw_boundary_controls(client, project, bundle_bytes):
    """Observe real duplicate/BOM/header/aggregate/exact-bound outcomes without accepting or committing a preview."""
    retained=project_bytes(project);checks=[]
    require(b'"content_profile":' in bundle_bytes,"Native artifact lacks the expected closed bundle field")
    malformed=[("native_plain_duplicate_bundle_rejected",bundle_bytes.replace(b'"content_profile":',b'"content_profile":"index-and-hashes","content_profile":',1)),
               ("native_decoded_duplicate_bundle_rejected",bundle_bytes.replace(b'"content_profile":',b'"content_profile":"index-and-hashes","content_\\u0070rofile":',1)),
               ("native_chosen_BOM_rejected_without_stripping",b'\xef\xbb\xbf'+bundle_bytes)]
    for name,raw in malformed:
        try:
            client.prepare_bundle_import(raw,target_index_schema_version=1,acknowledge_index_replacement=True,idempotency_key=str(uuid.uuid4()))
        except WorkspaceError as failure:
            require(failure.payload.get("code")=="invalid-request","Malformed raw import returned an unexpected typed error")
        else:
            raise RuntimeError("Malformed raw bundle was accepted")
        require(project_bytes(project)==retained,"Rejected raw import changed fixture files")
        checks.append(name)
    require(len(bundle_bytes)<=1048496,"Native artifact does not fit the chosen-file bound")
    chosen=bundle_bytes+b' '*(1048496-len(bundle_bytes))
    exact=raw_wrapper(chosen);require(len(exact)==1048576,"Exact request arithmetic differs")
    status,response=raw_import(client,exact)
    require(status==200 and set(response)=={"validation","preview","replacement"},"Exact1MiB import did not return the wrapped preview")
    require(response["validation"]["state"]=="valid","Exact-bound projection is not valid")
    require(project_bytes(project)==retained,"Exact-bound preparation wrote before confirmation")
    checks.append("native_exact1MiB_whole_request_prepares_without_write")
    # The maintained wrapper refuses an excess file before transport; this is distinct from the HTTP controls below.
    try:
        client.prepare_bundle_import(chosen+b' ',target_index_schema_version=1,acknowledge_index_replacement=True,idempotency_key=str(uuid.uuid4()))
    except ValueError:
        pass
    else:
        raise RuntimeError("Maintained import accepted an excess chosen byte")
    checks.append("maintained_client_one_excess_chosen_byte_rejected")
    status,response=raw_import(client,b'',declared_length=1048577)
    require(status==413 and response.get("code")=="payload-too-large","Oversized declared Content-Length was not rejected")
    checks.append("native_declared_length_one_excess_rejected")
    status,response=raw_import(client,exact+b' ',chunked=True)
    require(status==413 and response.get("code")=="payload-too-large","Actual chunked1MiB+1 body was not rejected")
    require(project_bytes(project)==retained,"Body-bound rejection changed fixture files")
    checks.append("native_chunked_actual_body_one_excess_rejected")
    return checks


def no_downgrade_controls(client, project, supplied_bundle1):
    """Reject both current-index2 and supplied-index2 downgrade attempts while retaining all real files."""
    retained=project_bytes(project);supplied2=json.dumps(client.bundle_preview()["bundle"],ensure_ascii=False,separators=(",",":")).encode()
    checks=[]
    for name,raw in (("native_current_index2_to1_rejected",supplied_bundle1),("native_supplied_index2_to1_rejected",supplied2)):
        try:
            client.prepare_bundle_import(raw,target_index_schema_version=1,acknowledge_index_replacement=True,idempotency_key=str(uuid.uuid4()))
        except WorkspaceError as failure:
            require(failure.payload.get("code")=="invalid-request","Downgrade returned an unexpected typed error")
        else:
            raise RuntimeError("Index2 downgrade was accepted")
        require(project_bytes(project)==retained,"Rejected downgrade changed a source, index or output")
        checks.append(name)
    return checks

def campaign(forge):
    """Exercise actual maintained methods on isolated real confined files, then confirm readonly preservation."""
    checks=[]
    with tempfile.TemporaryDirectory(prefix="forge-s6-native-client-") as temporary:
        project=Path(temporary)
        sources={"policy.md":b"# Policy\n\nThe organization shall review access quarterly.\n","second.md":b"# Second policy\n\nThe organization shall preserve audit records.\n"}
        for path,content in sources.items():
            (project/path).write_bytes(content)
        index={"schema_version":"forge.workspace/1","label":"Synthetic sensitive project","resources":[
            {"key":"policy-key","role":"policy-source","path":"policy.md"},{"key":"second-key","role":"policy-source","path":"second.md"}]}
        (project/"forge.workspace.json").write_text(json.dumps(index,indent=2)+"\n",encoding="utf-8")
        initial_index=(project/"forge.workspace.json").read_bytes()
        with Workspace(forge,project,read_only=False,api_major=2) as client:
            observed_contract_version=require_metadata_version(client)
            bundle=client.bundle_preview()["bundle"]
            require(len(bundle["pins"])==2,"Complete supplied denominator differs")
            checks.append("actual_complete_bundle_query")
            operation=client.wait(client.prepare_bundle_export("metadata-export.json",acknowledge_sensitive_metadata=True,idempotency_key=str(uuid.uuid4())))
            require(operation["state"]=="succeeded","Metadata export preparation did not succeed")
            preview=operation["result"]["preview"]
            require(not (project/"metadata-export.json").exists(),"Preparation wrote an artifact before confirmation")
            require((project/"forge.workspace.json").read_bytes()==initial_index,"Export preparation changed the index")
            checks.append("export_preview_no_write")
            confirm(client,preview)
            downloaded=client.download_bundle_export(operation["operation_id"],expected_sha256=preview["exact_bytes_sha256"])
            require(downloaded==(project/"metadata-export.json").read_bytes(),"Downloaded artifact differs from exact confined output")
            require(json.loads(downloaded)==bundle,"Published metadata differs from complete preview bundle")
            checks.append("confirmed_export_exact_json_download")
            checks.extend(raw_boundary_controls(client,project,downloaded))
            response=client.prepare_bundle_import(downloaded,target_index_schema_version=2,acknowledge_index_replacement=True,idempotency_key=str(uuid.uuid4()))
            require(set(response)=={"validation","preview","replacement"},"Direct import response is not the closed wrapper")
            replacement=response["replacement"]
            require(replacement["previous_index"]==index,"Previous full membership differs")
            require(replacement["proposed_index"]["schema_version"]=="forge.workspace/2","Explicit migration target differs")
            require(replacement["proposed_index"]["resources"]==index["resources"],"Migration changed ordered registrations")
            require(replacement["removed_resource_keys"]==[],"Unchanged membership produced removed keys")
            require((project/"forge.workspace.json").read_bytes()==initial_index,"Import preparation wrote before confirmation")
            checks.append("direct_import_full_replacement_no_write")
            confirm(client,response["preview"])
            require(json.loads((project/"forge.workspace.json").read_bytes())==replacement["proposed_index"],"Confirmed index differs from complete replacement")
            require(all((project/path).read_bytes()==content for path,content in sources.items()),"Index replacement changed a source")
            checks.append("confirmed_index_only_migration_source_preservation")
            checks.extend(no_downgrade_controls(client,project,downloaded))
            shutdown(client)
        retained={path.name:path.read_bytes() for path in project.iterdir() if path.is_file()}
        with Workspace(forge,project,read_only=True,api_major=2) as client:
            require(require_metadata_version(client)==observed_contract_version,"Readonly session negotiated a different metadata contract version")
            require(client.bundle_preview()["bundle"]["index"]["schema_version"]=="forge.workspace/2","Readonly query lost index2 pairing")
            for preparation in (lambda:client.prepare_bundle_export("readonly-output.json",acknowledge_sensitive_metadata=True,idempotency_key=str(uuid.uuid4())),
                                lambda:client.prepare_bundle_import(downloaded,target_index_schema_version=2,acknowledge_index_replacement=True,idempotency_key=str(uuid.uuid4()))):
                try:
                    preparation()
                except WorkspaceError as failure:
                    require(failure.payload.get("code")=="read-only-session","Readonly preparation returned an unexpected error")
                else:
                    raise RuntimeError("Readonly preparation unexpectedly succeeded")
            checks.append("readonly_both_preparations_rejected")
            shutdown(client)
        require({path.name:path.read_bytes() for path in project.iterdir() if path.is_file()}==retained,"Readonly session changed files")
        checks.append("readonly_and_shutdown_preserve_files")
    return {"format":"forge-s6-native-client-proposed/3","status":"passed","contract_version":observed_contract_version,"checks":checks}


def main():
    """Run only an explicitly selected local Forge binary and print a bounded redacted outcome."""
    parser=argparse.ArgumentParser();parser.add_argument("--forge",required=True);args=parser.parse_args()
    try:
        result=campaign(args.forge)
    except Exception:
        print(json.dumps({"format":"forge-s6-native-client-proposed/3","status":"failed","reason":"S6 companion failed; inspect the private root job"}))
        return 1
    print(json.dumps(result));return 0


if __name__=="__main__":
    raise SystemExit(main())
