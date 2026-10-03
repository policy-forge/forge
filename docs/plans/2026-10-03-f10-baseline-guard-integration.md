# F10 baseline guard integration prerequisite

This source proposal reuses the one-epoch baseline guard from retained draft
`3b0b5a294810cf5a8b25d9f37c661c31d763b33f`, whose original basis is
`aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. It was prepared against the observed
current source at `8342b92f963031866cd40ceefea56418c00c9e2a`. The six existing target files match the frozen
`9cfc6d0714a4a21b19a11421a7581199899c42cb` snapshots; current shared CLI,
README and usage additions are preserved. This is a prerequisite for F10, not
its reviewed-risk export or plural-epoch implementation.

## Changed behavior

A native baseline must have exactly one result epoch. The prior baseline is
strictly parsed and validated against the existing OSCAL schema first. The
cardinality guard then refuses a plural result array before extracting any
snapshots or adding findings to the supplied review report. The normal CLI
reports invalid input with exit 2 before publishing an artifact or report,
including `--fail-on never`. Existing one-result comparisons remain supported.
No schema, new dependency, workspace API, registration role or authority changes.

See [Assessment Results](../assessment-results.md) for the current CLI contract.

## Verification to execute on integrated source

Run the existing assessment-results integration suite and all existing baseline
checks, then the complete locked suite, formatting and strict current Clippy.
The three added native tests use the actual model builder and declared captured context
fixtures. They schema-validate all four plural variants before testing refusal:
populated or conclusion-free additional epoch, each in both result orders.

The CLI negative test covers 32 combinations of those four variants, any/never
exit gate, absent/existing binary destination sentinels and file/stdout artifact
modes. It checks exit 2, no stdout artifact, no file/report publication or sentinel
change, and unchanged manifest, baseline and captured context bytes. The direct
API test checks four plural variants against the whole prepopulated report and
requires the established error type. The one-result control compares whole
artifact/report objects through the API and both CLI exit gates. These are three
registered tests, not 39 independently registered tests or authentic assessor
epochs. Optional test evidence records synthetic fixture bytes only when Root
sets a fresh evidence directory.

Adjacent Rustdoc is present on the nine selected named declarations: analyze,
extract_snapshots and the seven added test/helper declarations. The current
whole modules still have nine baseline and fifteen integration-file legacy
function omissions. CLI field documentation is separate. Static parsing and
byte checks are not compiler, runtime or coverage evidence; Root must execute
and bind the resulting integrated source before reporting a new pass.

## Preserved history and remaining work

The old dated plan and two old verification reports are retained byte-for-byte
in the TEMP proposal history. Their observations describe the retained draft,
not this integration or later platforms, and they are not copied as a new
verification claim. New integrated execution, coverage and hook receipts must
be recorded separately.

Full F10 still requires explicit kind/key/UUID/result/fingerprint selection into
the F07/F08 POA&M contract, without invented owners or dates, and complete bounded
per-epoch identity, context, output, report and baseline correspondence. The
one-epoch refusal remains until real plural semantics and workflow controls are
implemented. Independent interoperability, owner and assessor/pilot judgments,
Windows/platform qualification, dependency audit, release and final integrated
goal-wide documentation gates remain open. This proposal records no merge,
tracker closure, human approval or acceptance.
