# OSCAL Model Deep Research for Automated Security Policy Conversion

## Scope, versioning, and open parameters

The Open Security Controls Assessment Language (OSCAL) is a set of structured, hierarchical formats (XML/JSON/YAML) intended to standardize how security control information is represented across the control lifecycle: control publication, implementation, and assessment. citeturn1search6 This report focuses on the OSCAL models relevant to that lifecycle—Catalog, Profile, Component Definition, “Implementation” (interpreted here as the System Security Plan), and Assessment (Assessment Plan + Assessment Results + Plan of Action and Milestones). citeturn5search0turn5search4turn3search5turn3search6turn4search1turn4search3

A practical constraint for your app is that OSCAL is not “one document.” It is an ecosystem of interlinked document types, where later-layer artifacts reference earlier-layer artifacts (e.g., Profiles import controls; SSPs import Profiles; assessment artifacts import SSPs / APs). citeturn5search0turn5search2turn3search0turn4search1turn4search3

This report uses OSCAL v1.2.0 model documentation and schema references where possible (as indicated by the v1.2.0 model reference and format outline pages). citeturn3search0turn3search5turn3search7turn3search6turn4search2turn4search1turn4search3 Note: v1.2.0 introduces a Control Mapping model (not requested) but its presence is relevant to roadmap decisions for automated “policy → framework crosswalk” features. citeturn3search4

Several key product parameters were not specified and remain intentionally open in this report: target programming language and runtime, deployment model (local vs SaaS), user roles and permissioning, target compliance framework(s) (e.g., NIST SP 800-53 vs custom internal catalogs), expected document scales (tens vs tens of thousands of controls/requirements), and expected integration points (GRC tools, ticketing, CI/CD). These open parameters materially affect architecture choices, especially for parsing accuracy, traceability UX, and performance/security controls. citeturn1search6turn4search2turn5search0

## OSCAL architecture and cross-model traceability

OSCAL’s model set is commonly described as layered: a Control layer (Catalog + Profile), an Implementation layer (Component Definition + System Security Plan), and an Assessment layer (Assessment Plan + Assessment Results + POA&M). citeturn5search0turn5search4turn5search13turn3search6turn4search1turn4search3turn39view0 The most important integration fact for your app is that these layers are designed to be *linked* and *processed*, not treated as isolated exports. For example, the Profile is a prerequisite “bridge” artifact: controls “used” later must first be imported through a Profile baseline/tailoring step. citeturn5search0turn5search13turn5search2

image_group{"layout":"carousel","aspect_ratio":"16:9","query":["OSCAL layers and models diagram NIST","OSCAL catalog profile system security plan assessment plan results relationships"],"num_per_query":1}

A concise way to view the data flow (and therefore the core PRD traceability requirement) is:

```text
Policy source(s)
  └─> (optional) Custom Policy Catalog (Catalog model)
         └─> Policy Baseline / Tailoring (Profile model)
                ├─> Implemented via components (Component Definition model)
                └─> Implemented in a system context (System Security Plan model)
                       └─> Assessed (Assessment Plan -> Assessment Results)
                              └─> Remediated (POA&M)
```

This pipeline closely matches NIST’s guidance that Profiles select and tailor controls, while Implementation models describe how those controls are implemented, and Assessment models capture preparation and outcomes of assessment activities. citeturn5search0turn5search4turn5search2turn4search2turn4search1turn4search3

Two additional architecture facts matter for app design:

OSCAL uses the Metaschema framework to produce schemas and documentation that support consistent XML/JSON/YAML representations; this makes multi-format conversion feasible and testable, but also implies your app should treat “format conversion” as a first-class operation with strong validation gates. citeturn1search7turn1search2turn1search1turn1search0

Across models, “back matter” is structurally consistent and intended specifically for linked/attached resources (citations, supporting evidence, graphics, etc.), enabling a uniform approach to evidence management in your UX and storage layer. citeturn5search5turn5search3turn3search0

## Model definitions, schema elements, and relationships

This section defines the models you requested, emphasizing the schema elements that drive conversion logic, traceability, and validation.

### Comparative model table

| Layer | Model (root object) | Primary purpose | Key cross-document relationship | Most relevant “policy conversion” use |
|---|---|---|---|---|
| Control | Catalog (`catalog`) | Publish a structured set of controls (and their parts/parameters) in a machine-readable way | Source artifact for Profile selection/tailoring | Represent policy as a *custom control catalog* (internal requirements as “controls”) |
| Control | Profile (`profile`) | Select, organize, and tailor controls into a baseline | Imports control sources; resolution produces a “result catalog” | Represent policy scope/baseline + tailoring decisions (inclusions, exclusions, parameter settings) |
| Implementation | Component Definition (`component-definition`) | Describe how controls are implemented in reusable components/capabilities | Used as starting narrative for SSP; supports documentary components (policy/process) | Represent a policy or procedure as a “documentary component” with control implementations |
| Implementation | System Security Plan (`system-security-plan`) | Describe how controls are implemented for a specific system | Imports a Profile baseline; references components and inventory items | Represent system-specific implementations derived from policy + actual system facts |
| Assessment | Assessment Plan (`assessment-plan`) | Describe planned assessment activities, objectives, scope, and resources | Imports SSP | Generate assessor-ready plan templates derived from control baseline and policy intent |
| Assessment | Assessment Results (`assessment-results`) | Record what happened: performed activities, observations, findings, risks | Imports Assessment Plan; links back to reviewed controls/objectives | Capture evidence/observations for policy compliance; enable POA&M generation |
| Assessment | POA&M (`plan-of-action-and-milestones`) | Track remediations for identified issues and risks | Can import SSP; links to findings/observations/risks | Convert policy exceptions and remediation plans into structured remediation work items |

This table is derived from NIST’s conceptual descriptions of the layers/models and v1.2.0 model reference/format documentation (including root object names and model roles). citeturn5search0turn5search4turn5search13turn5search17turn3search0turn3search7turn3search6turn4search2turn4search1turn4search3turn39view0

### Catalog model

**Definition and role.** The Catalog model represents a collection of controls, intended to standardize control definitions from different sources so they can be searched/imported/exported in a common structure. citeturn5search17turn3search5 For your use case, the Catalog model is the cleanest “native” representation of policy *requirements* when you do not yet know (or do not want to commit to) a mapping to an external control framework: you can create a “policy catalog” where each policy requirement becomes a control (or a statement part within a control).

**Core schema elements (most conversion-relevant).** At v1.2.0, the JSON outline shows the Catalog root `catalog` includes required `uuid` and `metadata`, and `metadata` includes required `title`, `last-modified`, `version`, and `oscal-version`. citeturn3search0 These fields are mandatory for your generator, which implies your system must manage document identity/versioning as a first-class concept.

Catalog controls are commonly organized into groups and/or controls arrays (structure referenced in the outline and broader model reference). citeturn3search5turn3search0turn5search14 Practically, for policies, “groups” are a direct analog to policy chapters/sections, while “controls” are analogs to policy requirements.

**Internal structure of controls.** OSCAL controls are built from reusable constructs (properties, links, parts, parameters). In the SP 800-53 annotated example, NIST illustrates that references/citations are encoded in catalog back matter as resources, while control text is structured as parts and linked to those resources. citeturn5search12turn5search5 This is directly relevant to a policy converter: you should preserve policy citations and cross-references as explicit resources (back matter) instead of embedding them inside prose.

### Profile model

**Definition and role.** A Profile is a tailoring of controls from one or more catalogs, with mechanisms to select (`import`), restructure/merge (`merge`), and amend (`modify`) controls. citeturn5search18turn5search13turn5search0 NIST’s architecture guidance also emphasizes that later layers “must” come through a Profile selection step: controls used later must be imported by a Profile. citeturn5search0

**Profile resolution (algorithmically important).** NIST’s Profile Resolution guidance describes three main steps—import, merge, modify—and recommends they be executed in that order, noting the process works naturally as linear processing even if alternative approaches might be possible. citeturn5search2 For your PRD, this is a key requirement: your app either needs an internal Profile Resolution engine that conforms to this process, or must integrate with tooling that performs it.

**Schema elements that drive policy conversion UX.** The v1.2.0 Profile JSON format outline shows required `uuid` and `metadata` (with required `title`, `last-modified`, `version`, `oscal-version`) analogous to the Catalog. citeturn3search7turn3search0 The Profile format also includes selection and modification constructs (exposed in the Profile model’s index and tutorials), which should inform your UI design around: “what controls are in scope,” “what is tailored,” and “why.” citeturn3search1turn5search6turn5search2

### Component Definition model

**Definition and role.** The Component Definition model describes implementations of controls for reusable components or capabilities. Crucially for your scenario, it supports not only technical components (hardware/software) but also *documentary components* implemented via policy, procedure, or process. citeturn4search2turn4search2 NIST explicitly states that information provided by components can be used by consumers to provide starting narratives for documenting control implementations in an SSP. citeturn4search2 This is the strongest “official” bridge between policy text and implementation narratives: your app can convert policy documents into documentary components whose control-implementation narratives are then pulled forward into an SSP.

**Schema/constraints signals.** The v1.2.0 Component Definition JSON reference describes the model’s intent and the root `component-definition`, and also indicates constraints like uniqueness and indexing (e.g., component UUIDs, capability uniqueness). citeturn4search2turn3search3 For PRD: treat component UUID uniqueness as a cross-document invariant and ensure stable identifiers across edits.

**Practical modeling approach for policies.** A “policy” can be represented as a documentary component whose `control-implementations` map directly to baseline control IDs (if you have a baseline Profile) or to your custom policy catalog control IDs (if you are building an internal catalog-first workflow). This is consistent with NIST’s description that documentary components may describe the implementation of controls in policy/procedure/process. citeturn4search2turn5search4

### System Security Plan model

**Definition and role.** The SSP model describes implementation of controls for a specific system and is used to capture the information typically found in a system security plan (e.g., as in NIST SP 800-18), with the root object `system-security-plan`. citeturn3search6turn3search2 In a policy-to-OSCAL app, an SSP is usually not generated from policy text alone; it emerges from a combination of policy-derived documentary components and system-specific facts (inventory, boundaries, hosting, responsible roles).

**Implication for conversion design.** If your app’s goal is “convert security policies into OSCAL,” it should explicitly define whether it outputs:
- documentary components only (Component Definition artifacts), or
- an SSP template with placeholders + trace links to policy statements, or
- a fully populated SSP (which typically requires asset/system data sources beyond policy text).

This separation is consistent with the separation of concerns between documentary components and system-specific SSP content described in Implementation-layer guidance and the SSP model’s stated purpose. citeturn5search4turn4search2turn3search6

### Assessment models

**Assessment Plan.** The Assessment Plan model is used to describe information typically provided by an assessor when preparing for an assessment, with root `assessment-plan`. citeturn4search1turn5search0 From a policy-conversion perspective, the Assessment Plan is where policy statements/requirements can be transformed into structured “what to test” scaffolding—especially if your converter also produces control-to-test mapping guidance and evidence expectations.

**Assessment Results.** Assessment Results contain “what actually happened,” including constructs to specify reviewed controls/objectives and selection logic. NIST’s v1.2.0 assessment-results schema documentation includes guidance for reviewed-controls resolution logic, and it explicitly notes that Assessment Results can re-select controls/objectives from the baseline identified by the SSP. citeturn4search3turn4search4 This is highly relevant to an automated toolchain: your app can generate “planned tests” in the AP, but the SAR (assessment-results) must reflect actual assessed scope, which may diverge.

**POA&M.** The POA&M model includes required `poam-items` and provides linkages to related observations/risks/finding UUIDs as part of remediation workflows. citeturn39view0 For policy conversion, POA&M is the structured endpoint for exceptions, gaps, and remediation commitments that policies frequently discuss informally.

### Cross-cutting OSCAL constructs that should shape your internal canonical model

**Metadata and identity.** OSCAL documents consistently require metadata including a document UUID and versioning fields; NIST recommends UUID v4 as a document identifier and describes how identifiers should be managed across revisions, with consistency unless the underlying subject changes significantly. citeturn3search0turn5search8turn5search1 This suggests your app needs:
- stable IDs for “policy statements” (internal subject IDs), and
- separate document-instance IDs (OSCAL UUID) per export artifact and/or per revision.

**Back matter, resources, and evidence.** NIST highlights back matter as identical across models and encourages authors/tool developers to place supporting content there and reference it from model bodies. citeturn5search5turn5search3turn5search12 This implies a uniform “evidence and references subsystem” in your app: citations, uploaded policy PDFs, screenshots, and test outputs should be modeled as resources referenced by links rather than embedded as opaque remarks.

**Avoid misusing `remarks`.** NIST guidance in the complete model documentation explicitly warns that `remarks` should not store arbitrary data; instead use `prop` or `link` for additional data not formally represented. citeturn4search4 For your PRD, this is a major design constraint: the converter must not “dump JSON blobs” into remarks to preserve information; it must use explicit extension patterns.

## Mapping common security policy constructs to OSCAL

A security policy document usually mixes “normative requirements” (must/shall), “scope/applicability,” “roles/responsibilities,” “procedures,” “exceptions,” and “enforcement/auditability.” OSCAL can represent all of these, but not always in a single model; the representation depends on whether you treat policy content as controls (Catalog/Profile) or as implementations of controls (Component/SSP) or both. citeturn5search0turn5search17turn4search2turn3search6

### Two canonical mapping strategies

**Catalog-first strategy (policy as controls).** Convert policy requirements into a custom Catalog (controls), optionally create Profiles for different baselines (e.g., “Corporate baseline,” “Engineering baseline”), then later map these to implementations and assessments. This aligns with the Catalog’s purpose as a machine-readable set of controls and the Profile’s purpose as baseline selection/tailoring. citeturn5search17turn5search13turn5search6turn3search0turn3search7

**Component-first strategy (policy as documentary components).** Convert policy documents into a Component Definition where each policy is a documentary component that “implements” controls (either from an external catalog/profile baseline or from your own). This aligns with NIST’s explicit support for documentary components representing policy/process/procedure and their use as starting narratives for SSP implementation statements. citeturn4search2turn5search4

Your PRD should likely support both strategies, since users vary: some organizations want to treat their policy library as the authoritative requirement set (catalog-first), while others want to “map policy to controls” for compliance automation (component-first, baseline-driven). citeturn5search0turn4search2turn3search4

### Mapping table from common policy constructs to OSCAL structures

| Policy construct | Normalized internal representation | OSCAL target representation | Notes for converter |
|---|---|---|---|
| Policy title, owner, approvals, revision history | Document metadata + lifecycle events | `metadata` fields (title/version/last-modified) and optionally actions/parties/roles | Catalog/Profile outlines show required metadata fields; metadata tutorial covers UUID/version expectations. citeturn3search0turn3search7turn5search8 |
| Scope / applicability (systems, teams, data classes) | Applicability predicates (who/what/when) | `prop` annotations on controls/components; system scope in SSP; assessment scope in AP | Use `prop` rather than arbitrary `remarks` where possible. citeturn4search4turn5search3 |
| Normative requirement (“must/shall”) | Atomic requirement statement (actor, action, object, condition) | Catalog: control statement `part`; Component/SSP: implementation statement narrative | Catalog model is explicitly for representing controls; Component model supports documentary components (policy). citeturn5search17turn4search2turn3search0turn5search12 |
| Parameters (“within 30 days”, “at least annually”) | Parameter object (name, value domain, default, selected value) | Catalog `param` or `part`; Profile parameter setting (`set-parameters` / modify); assessment timing/tasks | Profile supports tailoring and setting parameter values. citeturn5search13turn5search2turn3search1 |
| Process/procedure steps | Structured procedure graph (steps, inputs, outputs) | Documentary component description + links/resources; optionally assessment activities/steps | AP/AR models use activities/steps/tasks concepts for assessment processes. citeturn4search1turn4search3turn5search3 |
| Exceptions / compensating controls | Exception record (scope, rationale, expiry, compensating measures) | Component/SSP: control implementation status + props; POA&M items for remediation | POA&M has required `poam-items` and linkages to observations/risks. citeturn39view0turn4search3 |
| Evidence expectations (logs, screenshots, configs) | Evidence requirement + collection method | Back matter resources + links; Assessment Plan “what to collect”; Assessment Results “what collected” | Back matter is emphasized for linked/attached resources and citations. citeturn5search5turn5search3turn4search3 |

This mapping is grounded in the intended purposes of the models and NIST’s guidance on structured extension, back matter usage, and profile/implementation/assessment workflow. citeturn5search0turn5search5turn5search3turn4search2turn4search1turn4search3turn39view0

## Conversion algorithms, edge cases, validation, and tooling

### End-to-end conversion algorithm

A rigorous converter should be designed as a deterministic, auditable pipeline with intermediate representations, not as a single “policy → OSCAL JSON blob” transformer. This aligns with OSCAL’s own structured processing expectations (notably Profile Resolution) and with the availability of schemas/tooling for validation at each stage. citeturn5search2turn1search1turn5search14turn1search7

A recommended high-level algorithm is:

**Ingest → Parse → Normalize → Map → Assemble → Validate → Resolve/Derive → Export**

1) **Ingest**: Accept policy sources (PDF/DOCX/Markdown, etc.) and extract a structural hierarchy (headings, numbered clauses, tables).

2) **Parse into atomic requirements**: Split compound statements into atomic units (“one requirement per unit”), preserving provenance (document section, paragraph index). This provenance later becomes evidence links and traceability anchors.

3) **Normalize into a canonical schema**: Create internal objects such as:
   - `PolicyDocument`
   - `PolicySection`
   - `PolicyRequirement` (atomic requirement)
   - `PolicyParameter` (time windows, thresholds)
   - `PolicyException`
   - `Citation` / `EvidencePointer`

4) **Map to OSCAL modeling strategy**:
   - If catalog-first: generate Catalog controls from `PolicyRequirement`; optionally generate Profiles per baseline (department/system).
   - If component-first: generate a documentary component per policy (or per policy section), whose control-implementations reference a baseline catalog/profile.

5) **Assemble OSCAL artifacts**: Populate required metadata fields and stable UUIDs.

6) **Validate artifacts**: Validate against generated XML/JSON schemas; enforce additional constraints (indexing, uniqueness) and internal business rules (e.g., no orphaned links). Metaschema-generated schemas are intended precisely for this. citeturn1search2turn1search7turn5search14turn3search0turn4search2turn4search1turn4search3

7) **Resolve Profiles (if Profiles generated or used)**: Execute Profile Resolution in the recommended import → merge → modify order to obtain a resolved catalog baseline for downstream implementations/assessments. citeturn5search2turn1search1turn1search0

8) **Export in required formats**: Produce JSON, XML, YAML outputs as needed; ensure that conversion preserves semantics and validate post-conversion. Tools like the NIST OSCAL CLI explicitly support conversion among XML/JSON/YAML and validation. citeturn1search1turn1search0turn1search6

### Profile Resolution as a first-class feature

NIST’s Profile Resolution guidance states that implementations are strongly recommended to execute import, merge, modify in that order, and it explains that importing nothing yields no resulting catalog. citeturn5search2 That statement implies two PRD-level requirements:

Your app must prevent “baseline generation” workflows that skip import/selection and still claim to produce a resolved baseline.

Your app should surface resolution determinism to users (e.g., show resolved output changes caused by merge/modify directives), because Profile Resolution is the canonical place where tailoring occurs.

Profile Resolution can be delegated to tooling: the NIST OSCAL CLI explicitly supports resolving OSCAL profiles. citeturn1search1turn1search0

### Edge cases and how OSCAL constraints should shape your handling

**Ambiguous “should” vs “must.”** Policies often mix normative and advisory language. OSCAL can represent both, but your converter should either:
- map only normative requirements into Catalog controls / implemented requirements, and preserve advisory text as guidance parts or links/resources, or
- represent both but tag them differently using properties (`prop`) for downstream filtering.

This aligns with NIST’s emphasis on using structured properties/links for additional semantics and not misusing remarks for arbitrary extra data. citeturn4search4turn5search3

**Parameter extraction and tailoring drift.** A policy statement like “rotate keys every 90 days” is meaningfully different from “rotate keys every 365 days.” If you treat that as a parameter, the Profile model is the canonical place to set/override parameter values for a selected baseline. citeturn5search13turn5search2turn5search6 Any UI that lets users edit these values must preserve traceability from the original policy text to the final parameter value.

**Identifier stability across revisions.** NIST’s identifier guidance requires managing identifiers across revisions, keeping consistency unless the subject truly changes. citeturn5search1 For your app, that means you need stable internal “requirement IDs” and stable OSCAL IDs/UUIDs for corresponding OSCAL elements; regeneration should be deliberate (e.g., when a requirement is substantively altered vs editorially changed).

**Evidence and references.** Policies often cite standards, internal procedures, or vendor documents. NIST’s guidance shows that citations and bibliographic references belong in back matter as resources and should be linked from within the body. citeturn5search12turn5search5turn5search3 This implies a converter should extract references into resources and not leave URLs/citations buried in prose where they become unqueryable.

### Validation and tooling

OSCAL’s schemas and reference documentation are generated via Metaschema (format-agnostic model definitions → XML/JSON schemas, documentation). citeturn1search7turn1search2turn3search5 For your app, this means schema validation can be treated as “the ground truth” for structural correctness, but you should also plan for additional semantic validation (uniqueness, traceable references, internal invariants).

The NIST OSCAL tooling ecosystem includes an OSCAL Java CLI (oscal-cli) that supports:
- converting OSCAL content between XML/JSON/YAML,
- validating OSCAL resources,
- resolving profiles,
- validating Metaschema definitions, and
- generating schemas from Metaschema. citeturn1search1turn1search0

NIST also provides release-testing guidance noting that OSCAL tooling supports validation, conversion, and operations like profile resolution, and that this tooling underpins schema generation and reference pages. citeturn1search4

Finally, NIST publishes a dedicated examples repository with OSCAL example content in XML/JSON/YAML that is maintained via CI/CD conversion pipelines; this is valuable for “golden file” testing of your exporter and for UX examples. citeturn1search13

## PRD-oriented requirements for app workflows, APIs, storage, security, QA, and sample outputs

### UX and workflow requirements driven by OSCAL semantics

A policy-to-OSCAL app should not frame its workflow as “export JSON.” If it does, it will fail in usability once users need provenance, tailoring, and validation. Instead, the workflow should track OSCAL-native milestones:

**Ingestion and structuring.** Users need to see the parsed policy outline (sections/clauses) with a stable internal ID per clause (mapped later to control IDs/UUIDs). This supports identifier consistency across revisions. citeturn5search1

**Requirement atomization UI.** Provide an editing experience that lets users split/merge extracted requirements while preserving provenance links to source text and avoiding inadvertent identifier churn (e.g., warn when a split changes stable IDs). citeturn5search1

**Mapping strategy selection.** Offer explicit project configuration: “Represent policy as a Catalog baseline” vs “Represent policy as documentary Components mapped to an external baseline.” This maps directly to the intended roles of Catalog/Profile vs Component/SSP. citeturn5search17turn4search2turn5search0

**Tailoring and resolution visibility.** If Profiles are used, users need:
- a selection UI (what’s included/excluded),
- a tailoring UI (what’s modified, and why),
- and a “resolved output preview” UI to see the computed baseline.

This is driven by NIST’s Profile Resolution model and the recommended import→merge→modify process. citeturn5search2turn5search6turn1search1

**Evidence/resource manager.** Implement a consistent “resource library” UX backed by back matter resources and links so that citations/evidence can be reused across artifacts. NIST emphasizes back matter as consistent across models and intended for linked/attached resources. citeturn5search5turn5search3turn5search12

**Validation-first publishing.** The publish/export action should run schema validation and show actionable errors. This is consistent with NIST’s tooling emphasis on validate/convert operations and Metaschema-based schema generation. citeturn1search1turn1search0turn1search2turn5search14

### Internal data model and API surface

To avoid coupling your internal representation to a single OSCAL JSON shape, treat OSCAL as an output serialization of a canonical, typed domain model.

A pragmatic internal domain model (conceptual, not language-specific) is:

- `DocumentSource` (file, URL, repository commit)
- `PolicyDocument` (id, title, version, sections, metadata)
- `PolicyRequirement` (stable_id, text, modality, applicability_predicate, parameters, exceptions, references)
- `OSCALArtifact` (type, oscal_version, uuid, metadata, content_graph, serialization_formats)
- `TraceLink` (from_policy_requirement_id → oscal_json_path(s) + oscal_identifier(s))
- `Resource` (evidence/citation objects suitable for back matter)
- `ValidationResult` (schema errors + semantic errors)

This model supports NIST’s guidance to use structured props/links/resources rather than abusing remarks for arbitrary extra data, since you can map custom information into `prop`/`link` patterns and back matter resources. citeturn4search4turn5search3turn5search5

An API surface (REST or equivalent) that aligns with the workflow could expose endpoints such as:
- `POST /sources` (ingest policy)
- `POST /documents/{id}/parse`
- `POST /documents/{id}/requirements/normalize`
- `POST /projects/{id}/artifacts/generate` (catalog/profile/component/ssp/ap/ar/poam)
- `POST /artifacts/{id}/validate`
- `POST /profiles/{id}/resolve`
- `GET /trace/{requirement_id}` (traceability drilldown)
- `GET /artifacts/{id}/export?format=json|xml|yaml`

Even if you don’t implement these exact endpoints, the underlying operations correspond directly to OSCAL’s model operations (especially profile resolution) and to tooling capabilities (validate/convert/resolve). citeturn5search2turn1search1turn1search0

### Storage and serialization options

OSCAL is intentionally multi-format (XML/JSON/YAML) and NIST tooling supports converting among these formats. citeturn1search6turn1search1turn1search2 For storage, you typically have three viable approaches:

**Canonical internal graph + deterministic exporters.** Store normalized policy + mapping + trace graph internally, then generate OSCAL artifacts on demand. This is best for auditability and avoids “editing generated OSCAL by hand.”

**Canonical OSCAL JSON storage + convert outward.** Store OSCAL JSON as the canonical persisted artifact and use tooling to convert to XML/YAML (with validation gates). This leverages readily available JSON processing and aligns with oscal-cli conversion support. citeturn1search1turn1search0

**Document store with immutable revisions + derived projections.** Store each OSCAL artifact revision as immutable content (object storage), index key fields (uuid, control-id, requirement ids) for search and trace queries.

For evidence references in back matter, NIST’s extension tutorial describes how links reference back-matter resources by UUID and how resources can include `rlinks` to external content and optional hashes for integrity. citeturn5search3turn5search5 This supports an architecture where large binary evidence is stored externally, while OSCAL stores stable references + integrity metadata.

### Performance and security considerations

**Performance scaling risks.** Profile resolution and large artifact generation can be computationally expensive at scale, especially when:
- many catalogs are imported into profiles,
- many tailoring operations apply, or
- many requirements are mapped to many controls.

NIST’s profile resolution guidance is inherently iterative and ordered; implementations should expect linear processing with potential intermediate states. citeturn5search2 Your PRD should include explicit non-functional requirements for resolution time and caching strategies (e.g., cache resolved catalogs keyed by profile UUID+version).

**Security and privacy.** Policy documents can contain sensitive operational details or internal control weaknesses. Your app should treat ingested policy text and generated implementation narratives as sensitive content. While OSCAL itself does not prescribe security controls for the tooling, NIST’s architecture encourages structuring citations/evidence explicitly; that means your system will routinely handle sensitive evidence objects (logs, screenshots) referenced as resources. citeturn5search5turn5search3turn4search3 Design implications include encryption at rest, strong access controls, audit logs, and redaction workflows for exports.

**Data integrity for evidence.** Because back matter resources can include hashes (advanced but supported), a high-assurance deployment can optionally hash referenced evidence so that later assessments can verify integrity. citeturn5search3turn1search10

### Testing and QA plan

A robust QA strategy for a policy-to-OSCAL converter should combine schema validation with behavioral testing:

**Schema validation tests.** Each generated artifact is validated against the authoritative schemas produced from Metaschema, reflecting NIST’s intended use of generated schemas for well-formedness and model consistency. citeturn1search2turn1search7turn5search14

**Golden file regression tests.** Maintain canonical test fixtures: small policies → expected OSCAL outputs. The NIST oscal-content examples repository is a strong reference for formatting conventions and multi-format conversion expectations. citeturn1search13turn1search1

**Round-trip conversion tests.** Use oscal-cli to convert generated JSON → XML → JSON and ensure semantic equivalence (allowing for ordering differences). This directly leverages tool support for conversion and validation. citeturn1search1turn1search0

**Profile resolution conformance tests.** For any workflow that produces or modifies profiles, test that resolved catalogs match expected results and that import→merge→modify ordering is followed. citeturn5search2turn1search1

**Traceability tests.** Every `PolicyRequirement` must have trace links to:
- its OSCAL control/control-part (catalog-first) or
- its implemented requirement narrative (component/SSP) and, if applicable,
- its planned test (AP) and observed results (SAR/POA&M).

Traceability is essential because profile resolution and assessment artifacts can re-select controls/objectives from baselines; you must ensure your links remain stable even when scope changes. citeturn4search3turn5search1turn5search2

### Sample OSCAL outputs

The following examples are intentionally small and illustrative; they focus on showing how common policy statements can be structured into OSCAL artifacts and linked across layers. Required metadata fields (uuid/title/version/last-modified/oscal-version) are included consistent with v1.2.0 outlines. citeturn3search0turn3search7turn5search8

#### Sample policy catalog

```json
{
  "catalog": {
    "uuid": "11111111-1111-4111-8111-111111111111",
    "metadata": {
      "title": "Example Corporate Security Policy Catalog",
      "last-modified": "2026-02-10T00:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "groups": [
      {
        "id": "access-control",
        "title": "Access Control Policies",
        "controls": [
          {
            "id": "POL-AC-001",
            "title": "Multi-factor authentication for privileged access",
            "parts": [
              {
                "id": "POL-AC-001_smt",
                "name": "statement",
                "prose": "Privileged access to production systems requires multi-factor authentication."
              }
            ]
          }
        ]
      }
    ]
  }
}
```

#### Sample profile selecting the policy baseline

```json
{
  "profile": {
    "uuid": "22222222-2222-4222-8222-222222222222",
    "metadata": {
      "title": "Example Baseline Profile (Privileged Access)",
      "last-modified": "2026-02-10T00:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "imports": [
      {
        "href": "./policy-catalog.json",
        "include-controls": [
          {
            "with-ids": ["POL-AC-001"],
            "with-child-controls": "yes"
          }
        ]
      }
    ]
  }
}
```

Profiles are the canonical mechanism for selecting and tailoring controls into a baseline, and NIST recommends resolving profiles via import→merge→modify processing. citeturn5search13turn5search2turn1search1

#### Sample component definition treating policy as a documentary component

```json
{
  "component-definition": {
    "uuid": "33333333-3333-4333-8333-333333333333",
    "metadata": {
      "title": "Example Documentary Component: Corporate Access Control Policy",
      "last-modified": "2026-02-10T00:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "components": [
      {
        "uuid": "44444444-4444-4444-8444-444444444444",
        "type": "policy",
        "title": "Corporate Access Control Policy",
        "description": "A documentary component representing requirements for privileged access management.",
        "control-implementations": [
          {
            "uuid": "55555555-5555-4555-8555-555555555555",
            "source": "./resolved-profile.json",
            "description": "Implementation narratives derived from corporate policy.",
            "implemented-requirements": [
              {
                "uuid": "66666666-6666-4666-8666-666666666666",
                "control-id": "POL-AC-001",
                "description": "Policy requires MFA for privileged access to production systems."
              }
            ]
          }
        ]
      }
    ]
  }
}
```

This example follows NIST’s description that the Component Definition model can represent documentary components (policy/procedure/process) and can serve as a starting point for SSP control implementation narratives. citeturn4search2turn5search4

#### Sample assessment plan skeleton importing an SSP

```json
{
  "assessment-plan": {
    "uuid": "77777777-7777-4777-8777-777777777777",
    "metadata": {
      "title": "Example Assessment Plan (Policy-Derived)",
      "last-modified": "2026-02-10T00:00:00Z",
      "version": "0.1.0",
      "oscal-version": "1.2.0"
    },
    "import-ssp": {
      "href": "./ssp.json"
    },
    "reviewed-controls": {
      "control-selections": [
        {
          "include-controls": [
            { "control-id": "POL-AC-001" }
          ]
        }
      ]
    },
    "assessment-subjects": [
      {
        "type": "component",
        "include-subjects": [
          {
            "subject-uuid": "44444444-4444-4444-8444-444444444444",
            "type": "component"
          }
        ]
      }
    ],
    "tasks": [
      {
        "uuid": "88888888-8888-4888-8888-888888888888",
        "type": "interview",
        "title": "Validate MFA enforcement for privileged access",
        "description": "Confirm privileged access paths and verify MFA enforcement evidence for production systems."
      }
    ]
  }
}
```

The Assessment Plan is intended to capture assessor-provided information for preparing an assessment, and the Assessment Results/Plan models include reviewed-controls selection constructs with defined processing expectations. citeturn4search1turn4search3turn4search4

---

**Primary sources used.** This report prioritizes official NIST OSCAL model documentation, conceptual guidance, and schemas generated via Metaschema, plus the official tooling pages (oscal-cli) and NIST-maintained examples repositories hosted on entity["company","GitHub","code hosting platform"]. citeturn1search6turn5search0turn3search5turn4search2turn1search1turn1search13turn1search16