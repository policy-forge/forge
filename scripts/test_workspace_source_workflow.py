#!/usr/bin/env python3
"""Verify real API2.3 source export, durable restore and fresh-session lookup on owned Unix fixtures."""
from pathlib import Path
import argparse
import datetime,hashlib,http.client,json,os,sys,tempfile,time,uuid
from workspace_client import Workspace,WorkspaceError
root=Path(__file__).resolve().parent.parent

def pin(path):
    """Record only exact synthetic source/binary hashes, never credentials or source excerpts."""
    raw=path.read_bytes();return {'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
def key():
    """Use a fresh server-admitted request identity without retaining capability secrets."""
    return str(uuid.uuid4())
def write_index(path,label,resources):
    """Create one actual explicit index2; discovery remains outside this workflow."""
    value={'schema_version':'forge.workspace/2','label':label,'resources':resources}
    (path/'forge.workspace.json').write_text(json.dumps(value,separators=(',',':'))+'\n')
def refusal(action,code):
    """Verify a real typed refusal without rendering a private transport envelope."""
    try:action()
    except WorkspaceError as error:assert error.payload['code']==code;return code
    raise AssertionError('Expected real typed refusal '+code)

def shutdown(client):
    """Require the authenticated response and native zero exit before ordinary client cleanup."""
    assert client.request("POST",client.api_path("/session/shutdown"),{})=={"state":"shutting-down"}
    assert client.process.wait(timeout=5)==0

def run(args):
    """Bind an existing native binary and retain a private receipt for every actual source-flow result."""
    if os.name!="posix" or not __debug__:
        raise RuntimeError("Native Unix source controls require POSIX and active assertions")
    out=Path(args.out).resolve();out.mkdir(mode=0o700)
    binary=Path(args.forge).resolve(strict=True)
    checks=[]
    receipt=None
    binary_before=pin(binary)
    receipt={'format':'forge.s6-real-source-workflow/1','started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'binary':binary_before,
        'source_client':pin(root/'scripts/workspace_client.py'),'source_workflow':pin(Path(__file__)),'normal_session_exits':0,'checks':checks,'platform':sys.platform,'qualification':'actual native Unix binary and maintained client over authenticated HTTP; no browser/Windows/power-loss/AT/human acceptance'}
    try:
        with tempfile.TemporaryDirectory(prefix='forge-source-donor-') as donor_name,tempfile.TemporaryDirectory(prefix='forge-source-recipient-') as recipient_name:
            donor=Path(donor_name);recipient=Path(recipient_name)
            (donor/'policies').mkdir();(donor/'evidence').mkdir()
            source=b'# Policy\n\n## Rule\nThe service must restrict access.\n';opaque=b'\x00\xff\x80\nexact opaque source\x00'
            (donor/'policies/access.md').write_bytes(source);(donor/'evidence/source.bin').write_bytes(opaque)
            resources=[{'key':'policy','role':'policy-source','path':'policies/access.md'},{'key':'fingerprint','role':'lifecycle-source','path':'evidence/source.bin'}]
            write_index(donor,'Synthetic source donor',resources)
            (recipient/'local.md').write_bytes(b'# Local\n\n## Rule\nRetained unregistered source.\n');(recipient/'leave.bin').write_bytes(b'removed registration file retained')
            write_index(recipient,'Synthetic recipient',[{'key':'local','role':'policy-source','path':'local.md'},{'key':'removed','role':'lifecycle-source','path':'leave.bin'}])
            donor_inputs={name:pin(donor/name) for name in ['forge.workspace.json','policies/access.md','evidence/source.bin']}
            old_recipient={name:pin(recipient/name) for name in ['forge.workspace.json','local.md','leave.bin']}
            with Workspace(binary,donor,read_only=False,api_major=2) as client:
                assert client._contract_version in ('2.3.0','2.4.0');receipt['api_version']=client._contract_version;export_key=key()
                operation=client.prepare_source_bundle_export('source.json',acknowledge_sensitive_metadata=True,acknowledge_source_content=True,idempotency_key=export_key)
                replay=client.prepare_source_bundle_export('source.json',acknowledge_sensitive_metadata=True,acknowledge_source_content=True,idempotency_key=export_key)
                assert replay==operation;checks.append('source-export-original202-replay')
                completed=client.wait(operation);receipt['export_outcome']={field:completed.get(field) for field in ['state','error']};assert completed['state']=='succeeded';preview=completed['result']['preview']
                assert not (donor/'source.json').exists();checks.append('source-export-preview-no-write')
                refusal(lambda:client.download_source_bundle_export(operation['operation_id'],expected_sha256=preview['exact_bytes_sha256']),'not-found')
                committed=client.commit(preview,confirmed=True,idempotency_key=key());assert committed['state']=='succeeded'
                bundle=client.download_source_bundle_export(operation['operation_id'],expected_sha256=preview['exact_bytes_sha256'])
                assert bundle==(donor/'source.json').read_bytes();checks.append('source-export-exact-committed-download')
                connection=http.client.HTTPConnection('127.0.0.1',client._port,timeout=10)
                connection.request('GET',client.api_path('/project/source-bundle-exports/'+operation['operation_id']+'/download'),headers={'Authorization':'Bearer '+client._capability})
                response=connection.getresponse();assert response.status==200
                assert response.getheader('Content-Disposition')=='attachment; filename="forge-workspace-index-and-source-content.json"'
                assert response.getheader('Content-Type')=='application/json';assert response.getheader('Cache-Control')=='no-store'
                assert response.read(1048430)==bundle;connection.close();checks.append('source-download-normative-header-and-exact-bytes')
                refusal(lambda:client.request('GET',client.api_path('/project/bundle-exports/'+operation['operation_id']+'/download'),raw=True),'not-found')
                assert donor_inputs=={name:pin(donor/name) for name in donor_inputs};checks.append('private-source-family-and-original-inputs-preserved')
                shutdown(client)
                receipt['normal_session_exits']+=1
            with Workspace(binary,recipient,read_only=False,api_major=2) as client:
                import_key=key();arguments={'target_index_schema_version':2,'acknowledge_index_replacement':True,'acknowledge_source_content':True,'acknowledge_replace_files':True,'idempotency_key':import_key}
                result=client.prepare_source_bundle_import(bundle,**arguments);preview=result['preview']
                assert client.prepare_source_bundle_import(bundle,**arguments)==result
                assert old_recipient=={name:pin(recipient/name) for name in old_recipient}
                assert not (recipient/'policies').exists();assert not (recipient/'evidence').exists()
                assert len(preview['targets'])==3 and preview['targets'][-1]['kind']=='index'
                assert len(preview['directories'])==2;checks.append('complete-source-preview-replay-and-explicit-directories-no-write')
                operation_id=preview['operation_id'];refusal(lambda:client.source_restore_status(operation_id),'not-found')
                assert old_recipient=={name:pin(recipient/name) for name in old_recipient};checks.append('preallocated-id-before-acceptance-not-found')
                commit_key=key();accepted=client.commit_source_restore(preview,confirmed=True,idempotency_key=commit_key)
                assert accepted['operation_id']==operation_id and accepted['state']=='pending' and accepted['write_outcome']=='unmeasured'
                assert client.commit_source_restore(preview,confirmed=True,idempotency_key=commit_key)==accepted
                # A durable commit decision can truthfully succeed before native cleanup ends.
                # This workflow verifies settlement under the same original polling budget.
                settlement_deadline=time.monotonic()+65
                outcome=client.wait_source_restore(operation_id)
                receipt['cleanup_pending_observations']=0
                while outcome['state']=='succeeded' and outcome['cleanup_state']=='pending':
                    receipt['cleanup_pending_observations']+=1
                    remaining=settlement_deadline-time.monotonic()
                    if remaining<=0:
                        raise AssertionError('Source cleanup did not settle within the original polling budget')
                    time.sleep(min(0.01,remaining))
                    outcome=client.source_restore_status(operation_id)
                receipt['restore_outcome']={field:outcome[field] for field in ['state','write_outcome','cleanup_state']}
                receipt['restore_error_code']=(outcome.get('error') or {}).get('code')
                assert outcome['state']=='succeeded' and outcome['write_outcome']=='committed' and outcome['cleanup_state']=='verified'
                assert (recipient/'policies/access.md').read_bytes()==source and (recipient/'evidence/source.bin').read_bytes()==opaque
                index=json.loads((recipient/'forge.workspace.json').read_bytes());assert index['resources']==resources
                assert old_recipient['local.md']==pin(recipient/'local.md') and old_recipient['leave.bin']==pin(recipient/'leave.bin')
                assert client.cancel_source_restore(operation_id)['state']=='succeeded'
                checks.extend(['durable-original-pending202-and-one-time-replay','exact-source-and-complete-index-replacement','removed-registrations-retain-files','terminal-cancel-preserves-committed-facts'])
                shutdown(client)
                receipt['normal_session_exits']+=1
            with Workspace(binary,recipient,read_only=True,api_major=2) as client:
                reopened=client.source_restore_status(operation_id);assert reopened==outcome
                refusal(lambda:client.commit_source_restore(preview,confirmed=True,idempotency_key=key()),'read-only-session')
                refusal(lambda:client.cancel_source_restore(operation_id),'read-only-session')
                checks.append('fresh-same-root-read-only-outcome-no-old-authority')
                shutdown(client)
                receipt['normal_session_exits']+=1
            with Workspace(binary,recipient,read_only=False,api_major=2) as client:
                assert client.source_restore_status(operation_id)==outcome
                refusal(lambda:client.commit_source_restore(preview,confirmed=True,idempotency_key=key()),'not-found')
                checks.append('fresh-writable-session-does-not-revive-old-receipt')
                shutdown(client)
                receipt['normal_session_exits']+=1
            receipt['outcome']={name:outcome[name] for name in ['operation_id','state','write_outcome','cleanup_state']}
            receipt['artifact']={'sha256':hashlib.sha256(bundle).hexdigest(),'bytes':len(bundle)}
            receipt['status']='passed'
    except BaseException as error:
        receipt['status']='failed';receipt['failure_type']=type(error).__name__
        raise
    finally:
        assert pin(binary)==binary_before
        receipt['finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
        receipt['check_count']=len(checks)
        (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        print(json.dumps({'receipt':str(out/'receipt.json'),'status':receipt.get('status'),'check_count':len(checks)}))

    return 0 if receipt['status']=='passed' else 1


def main():
    """Require explicit native build and new private output inputs without provisioning tools."""
    parser=argparse.ArgumentParser()
    parser.add_argument('--forge',required=True)
    parser.add_argument('--out',required=True)
    return run(parser.parse_args())

if __name__=='__main__':
    raise SystemExit(main())
