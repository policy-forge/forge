# Maintained-client failure observation

The workspace verifier can retain a separate, small diagnostic when the maintained client fails. The original `forge.workspace-verification/2` and `forge.workspace-client-verification/2` contracts remain the sources for suite status and counts. The diagnostic helps distinguish a recognized conformance banner, a recognized receipt-publication banner and an unavailable or unrecognized observation.

The fixed file is `workspace-client-failure-observation.json`. Its closed `forge.workspace-client-failure-observation/1` object has six fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Fixed diagnostic contract identifier. |
| `producer_exit_code` | Original integer exit, or null when no exit was observed. |
| `subprocess_failure` | Null or an existing fixed subprocess-failure code. |
| `banner_outcome` | `conformance-failed`, `publication-failed`, `unknown` or `unavailable`. |
| `inner_receipt_fact` | `closed-client-conformance-failed`, `nonfailed-not-authority`, `not-found` or `invalid-or-unreadable`. |
| `identity` | The tested commit and original hash/size observations for the wrapper, maintained-client producer and client library. |

The diagnostic is created after the maintained-client row has already failed, while its private receipt still exists. It publishes at most 2 KiB through the existing no-replacement receipt writer. Existing destinations are preserved. A diagnostic fault cannot change the primary failure, skip later selected suites or suppress the final input recheck.

Only complete recognized LF or CRLF banners receive a named classification. Mixed or arbitrary output remains unknown, and capture failures remain unavailable. The private receipt read is bounded to 4 MiB and rejects unsafe kinds, changed generations, duplicate keys and nonfinite JSON. A closed failed receipt has the producer's exact existing failed shape. `nonfailed-not-authority` records only a passed-looking header and field set; it validates no passed counts or operation outcomes. A failed producer stays failed regardless of that fact.

The file contains no raw subprocess output, private receipt contents, arbitrary exception text, credentials or project/native paths. Original input hashes are before-capture observations. The outer receipt still determines whether its final input recheck succeeded. Point-in-time observations do not establish an atomic snapshot or the cause of an earlier run whose private receipt is gone.

## Hosted upload

The verifier's `main` passes a fixed publication callback to the failed-client path. After a successful new bounded publication, it can append `client_failure_observation_published=true` to the runner's qualified regular `GITHUB_OUTPUT` file. The separate hosted upload requires that exact step output and a failed verifier outcome. File existence alone grants no upload permission, so a pre-existing, oversized or refused file cannot authorize upload.

Missing or unsafe output files, pre-write faults and short writes fail the optional flag operation. A complete flag already written cannot be retracted after a later close or path-check fault; its prerequisite is still successful new bounded publication. This observation remains best effort. The original singleton outer-receipt upload is preserved.

## Reproduction and interpretation

The current [API verification guide](plans/2026-10-02-f04-current-api-verification.md) describes the build, three-suite command, checkout bindings and original exit codes. Use a fresh empty output directory for each attempt. The separate diagnostic appears only on a failed maintained-client row when publication succeeds; a local run can produce the file without a hosted step-output file.

Run the standard-library controls from a full checkout:

```sh
python3 -B scripts/test_verify_workspace.py
```

These controls include synthetic process observations and real temporary-file operations. They establish their declared failure and publication behaviors. They do not execute a new native Windows client, recover the destroyed private receipt from PR #204, prove the supported browser/platform matrix or provide owner acceptance.

The accompanying source-bound verification record reports measured Python line/call coverage, selected and whole docstrings, preserved original fixtures and the exact unchanged Rust coverage cohort. Unexecuted lines and functions remain explicit. The final integrated roadmap documentation review, dependency/security, accessibility, user/pilot, platform and release gates remain open.

The fixed runner-output line is written with binary mode where available, preserving its ASCII LF bytes on Windows as well as Unix. This is separate from classifying the client's exact LF or CRLF banners.
