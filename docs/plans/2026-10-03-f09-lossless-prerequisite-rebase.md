# F09 export prerequisite integration

The prerequisite is now applied to integration basis
`309b5e5f7b8265f5adec4e959152d26b44c3dfc9`, which includes the current F10
baseline guard. The capture and proposal history below remains separate from
current execution. See the [current verification guide](../export-preservation-verification.md)
and its [source-bound report](2026-10-03-f09-export-integration-verification.json)
for completed checks and exact documentation/coverage denominators.

This source proposal ports the bounded Catalog/Component JSON/YAML prerequisite
from draft `512046e26650088a3d7048facd4a285c915579e7` onto the current integration
stack captured at `8342b92f963031866cd40ceefea56418c00c9e2a`. The two changed source
files still equal the common base `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`.
README and usage documentation were subsequently rebased, with exact paragraph
guards, onto Root's working-tree doc successor while HEAD remained
`8342b92f963031866cd40ceefea56418c00c9e2a`. This records byte movement, not
a committed integration result. The first guarded check refusal is preserved
as history; it performed no managed write.
Current CLI, workspace, dependency and documentation successors are preserved;
this plan does not copy the old README or old usage guide over current files.

The prior export path constructs partial generation envelopes before validation.
This prerequisite validates and emits the complete original decoded JSON-compatible
Catalog or Component Definition tree for JSON/YAML input to JSON/YAML output.
It retains supported native fields and array order, rejects duplicate keys and
invalid schema data before publication, and keeps the pinned offline OSCAL 1.2.3
validator and supported declarations 1.2.0 through 1.2.3.

The rebase adds an export-only numeric admission rule selected by Root: a numeric
value must be represented exactly as i64 or u64. Every f64 representation is
refused, including `1.0`, `1e0`, native port `443.0`/`443e0`, high-precision decimals,
and JSON integer overflow decoded through floating point. Numeric strings retain
their string semantics. The pinned YAML decoder can turn an enormous untagged
integer-looking scalar into a string; this guard does not reinterpret strings or
claim universal numeric-token classification. Explicit `!!int` overflow remains
a decoder refusal. Shared strict parser consumers keep their existing rules.
The integer boundaries are −9,223,372,036,854,775,808 through 18,446,744,073,709,551,615;
model schema and semantic checks can impose additional restrictions. Wider or
floating numeric support needs a separately consumed and qualified contract.

The regular-file input reader retains its 50 MiB raw cap. Duplicate-safe JSON
and YAML parsing share the existing value visitor, with decoded depth capped at
100 and each decoded string/key capped at raw input length. Those checks run after
decoding and do not establish a hard allocation, time or expanded-alias memory
bound. YAML aliases may expand and its decoder can erase nonlocal URI-form tags.
JSON object ordering, original bytes, YAML comments/tags/anchors/presentation and
arbitrary numeric-token exactness are not preservation claims. XML input/output
and direct typed helper APIs remain partial model projections.

The proposed integration suite keeps all twenty original draft tests unchanged
and appends CLI integer-boundary and numeric-refusal cases. Rich synthetic
Catalog and Component fixtures contain native metadata, statements, capabilities,
protocol ranges and back-matter beyond the generation structs. Tests compare the
whole tree and array positions, exercise JSON→JSON and JSON→YAML→JSON, and guard
both an existing sentinel destination and an absent destination on rejection.
Three new unit controls exercise exact signed/unsigned decoder boundaries and
nested JSON/YAML refusal. Execution outcomes belong to the current verification guide and source-bound report;
this retained proposal description does not transfer historical test credit.
The signed-boundary parser fixture deliberately makes no schema-validity claim.
Source immutability assertions apply to stdout or distinct destinations: an
explicit `--output` naming the input retains the existing in-place rewrite behavior.

Root owns application, compilation, formatting, strict Clippy, focused and full
Rust tests, exact-source doc/coverage derivation, independent review, mandatory
hook, draft/publication/tracker readbacks and any hosted runs. Historical draft
verification bytes remain separate inputs and earn no new integrated runtime
credit. The original TEMP author lane executed no candidate, test, compiler or product.
Root performed the current integration runs recorded separately.

[PRD 060](../PRD/060-prd-evidence-implementation-linking.md) S-3 is still open:
actual overlay emission, supported models/representations, collision semantics,
evidence-byte exclusion, scope authorization, per-model whole-tree round trips,
independent interoperability, privacy/owner/pilot evidence and linkage/evidence
freshness are not discharged by this prerequisite. F09 numeric/interop scope,
XML fidelity and YAML decoder resource qualification remain open. All broader
roadmap Must/Should acceptance, platform, supply-chain, human/AT and release gates
remain separate. The final integrated documentation review is not closed here.
