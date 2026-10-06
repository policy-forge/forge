//! Typed native projection for the proposed author-supplied POA&M item subset.
//!
//! Native item links point to exact external AR objects. The projection deliberately
//! does not create native local finding/risk references without their full native
//! dependency closures. Namespaced properties carry authored workflow declarations;
//! independent-tool interpretation and approval of this subset remain open.

use std::io::Write as _;
use std::sync::OnceLock;

use serde::Serialize;
use uuid::Uuid;

use super::manifest::{ArtifactManifest, RoleManifest};
use super::workflow::{MAX_OUTPUT_BYTES, Milestone, Owner, WorkflowManifest};
use crate::ForgeError;

/// Stable namespace for explicit Forge workflow declaration properties; never fetched.
const NS: &str = "https://policy-forge.github.io/forge/ns/poam-workflow";
/// Offline validator for the unchanged official release-pinned native POA&M schema.
static VALIDATOR: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

/// Complete native root, constructed only after actual source and workflow validation.
#[derive(Serialize)]
struct NativeRoot {
    /// Exact standards-native POA&M root name.
    #[serde(rename = "plan-of-action-and-milestones")]
    document: NativePlan,
}

/// Supported native subset without invented observations, findings or risks.
#[derive(Serialize)]
struct NativePlan {
    /// Existing immutable plan-key UUID-v5 identity.
    uuid: String,
    /// Author's declared metadata, roles and parties.
    metadata: Metadata,
    /// Confined relative reference to the exact captured SSP companion.
    #[serde(rename = "import-ssp")]
    import_ssp: Import,
    /// Explicit nonempty selected author-supplied remediation items.
    #[serde(rename = "poam-items")]
    items: Vec<NativeItem>,
    /// Five exact external artifact resource/hash receipts, without embedded source content.
    #[serde(rename = "back-matter")]
    back_matter: BackMatter,
}

/// Exact author metadata; no hidden time, actors or version values are inferred.
#[derive(Serialize)]
struct Metadata {
    /// Author-supplied plan title.
    title: String,
    /// Author-supplied RFC3339 modification timestamp.
    #[serde(rename = "last-modified")]
    last_modified: String,
    /// Author-supplied document version.
    version: String,
    /// Release matched to the native vendored schema.
    #[serde(rename = "oscal-version")]
    oscal_version: &'static str,
    /// Declared same-plan role IDs and titles.
    roles: Vec<RoleManifest>,
    /// Declared same-plan parties with stable domain-separated IDs.
    parties: Vec<NativeParty>,
    /// Explicit source and assertion-boundary metadata.
    props: Vec<Property>,
}

/// Stable party identity from the author's exact party key within the plan.
#[derive(Serialize)]
struct NativeParty {
    /// Domain-separated plan/party UUID-v5 identity.
    uuid: String,
    /// Original explicit person/organization declaration.
    #[serde(rename = "type")]
    party_type: &'static str,
    /// Original author-supplied party name.
    name: String,
    /// Exact stable key retained for workflow interpretation.
    props: Vec<Property>,
}

/// Standards-native external SSP reference, not a copied or modified SSP.
#[derive(Serialize)]
struct Import {
    /// Percent-encoded confined companion path relative to the bundle root.
    href: String,
}

/// Explicit native POA&M item with original authored prose and exact source links.
#[derive(Serialize)]
struct NativeItem {
    /// Existing key-scoped item UUID-v5 identity.
    uuid: String,
    /// Author-supplied title.
    title: String,
    /// Author-supplied work description, not copied assessment prose.
    description: String,
    /// Closed known namespaced property projection of complete authored workflow records.
    props: Vec<Property>,
    /// External source links; no dangling native local related-finding/risk IDs.
    links: Vec<Link>,
}

/// Explicit responsibility projection with both exact author keys and native party identity.
#[derive(Serialize)]
struct NativeOwner<'a> {
    /// Same-plan declared role identifier.
    role_id: &'a str,
    /// Original exact stable party key, not an authenticated account.
    party_key: &'a str,
    /// Same-plan native party UUID derived independently of mutable names.
    party_uuid: String,
    /// Author-supplied assignment rationale.
    rationale: &'a str,
}

/// Complete milestone declaration with an explicit stable native identity in the property subset.
#[derive(Serialize)]
struct NativeMilestone<'a> {
    /// Existing immutable plan/item/milestone UUID-v5 identity.
    uuid: String,
    /// All original closed milestone fields, including owners, dates, dependencies and history.
    #[serde(flatten)]
    declaration: &'a Milestone,
}

/// Explicit property in the fixed Forge workflow namespace.
#[derive(Serialize)]
struct Property {
    /// Fixed known property name, never an author-controlled native key.
    name: &'static str,
    /// Fixed namespace URI; no lookup is performed.
    ns: &'static str,
    /// Exact scalar or deterministic JSON declaration value.
    value: String,
}

/// External exact AR object link; full source tuple stays in the associated property.
#[derive(Serialize)]
struct Link {
    /// Percent-encoded confined AR path plus original canonical object UUID fragment.
    href: String,
    /// Fixed relation distinguishing source provenance from remediation verification.
    rel: &'static str,
}

/// Native back-matter container of exact source file receipts only.
#[derive(Serialize)]
struct BackMatter {
    /// Complete AR/AP/SSP/Profile/Catalog resource pins in fixed domain order.
    resources: Vec<Resource>,
}

/// Exact original file receipt, without evidence/source content embedding.
#[derive(Serialize)]
struct Resource {
    /// Domain-separated plan/source-kind UUID-v5 receipt identity.
    uuid: String,
    /// Fixed source kind label, not raw sensitive prose.
    title: &'static str,
    /// One relative local source link carrying the actual validated file digest.
    rlinks: Vec<ResourceLink>,
}

/// One exact external source file and hash receipt.
#[derive(Serialize)]
struct ResourceLink {
    /// Percent-encoded normalized relative artifact path.
    href: String,
    /// Explicit JSON media type for the supported source profile.
    #[serde(rename = "media-type")]
    media_type: &'static str,
    /// Exact expected digest already matched by F07 capture.
    hashes: Vec<Hash>,
}

/// Native SHA256 receipt, distinct from a canonical selected-object hash.
#[derive(Serialize)]
struct Hash {
    /// Fixed native SHA256 algorithm label.
    algorithm: &'static str,
    /// Exact lower-case whole-file digest matched by actual source capture.
    value: String,
}

/// Construct through typed structures, bound serialization and validate the entire native JSON.
pub(super) fn render(manifest: &WorkflowManifest) -> Result<Vec<u8>, ForgeError> {
    let mut items: Vec<_> = manifest.items.iter().collect();
    items.sort_by(|a, b| a.key.cmp(&b.key));
    let artifact = relative_uri(&manifest.source.assessment_results.artifact)?;
    let mut native_items = Vec::with_capacity(items.len());
    for item in items {
        native_items.push(NativeItem {
            uuid: super::identity::item(&manifest.document.key, &item.key).to_string(),
            title: item.title.clone(),
            description: item.description.clone(),
            props: vec![
                property("stable-key", item.key.clone()),
                property("source-references", declaration(&item.source_refs)?),
                property(
                    "owners",
                    declaration(&native_owners(&manifest.document.key, &item.owners))?,
                ),
                property("target-date", item.target_date.clone()),
                property("state", item.state.as_str().to_string()),
                property("history", declaration(&item.history)?),
                property(
                    "milestones",
                    declaration(
                        &item
                            .milestones
                            .iter()
                            .map(|step| NativeMilestone {
                                uuid: super::identity::milestone(
                                    &manifest.document.key,
                                    &item.key,
                                    &step.key,
                                )
                                .to_string(),
                                declaration: step,
                            })
                            .collect::<Vec<_>>(),
                    )?,
                ),
            ],
            links: item
                .source_refs
                .iter()
                .map(|reference| Link {
                    href: format!("{artifact}#{}", reference.uuid),
                    rel: "assessment-source",
                })
                .collect(),
        });
    }
    let mut parties: Vec<_> = manifest.parties.iter().collect();
    parties.sort_by(|a, b| a.key.cmp(&b.key));
    let mut roles = manifest.roles.clone();
    roles.sort_by(|a, b| a.id.cmp(&b.id));
    let native = NativeRoot { document: NativePlan {
        uuid: super::identity::document(&manifest.document.key).to_string(),
        metadata: Metadata { title: manifest.document.title.clone(),
            last_modified: manifest.document.last_modified.clone(), version: manifest.document.version.clone(),
            oscal_version: "1.2.3", roles,
            parties: parties.into_iter().map(|party| NativeParty {
                uuid: scoped_uuid(&manifest.document.key, "party", &party.key).to_string(),
                party_type: match party.party_type {
                    crate::assessment_results::manifest::PartyType::Person => "person",
                    crate::assessment_results::manifest::PartyType::Organization => "organization",
                }, name: party.name.clone(), props: vec![property("stable-key", party.key.clone())],
            }).collect(),
            props: vec![property("manifest-schema", manifest.schema_version.clone()),
                property("source-result-uuid", manifest.source.result.uuid.clone()),
                property("source-result-key", manifest.source.result.key.clone()),
                property("source-file-sha256", manifest.source.assessment_results.expected_sha256.clone()),
                property("assertion-boundary", "Declared remediation and review assertions; effectiveness, evidence freshness and actor authority unverified.".to_string())],
        }, import_ssp: Import { href: relative_uri(&manifest.source.context.ssp.artifact)? },
        items: native_items, back_matter: source_resources(manifest)?,
    }};
    let bytes = json_line(&native)?;
    validate_native(&bytes)?;
    Ok(bytes)
}

/// Correlate each exact authored responsibility with its declared native party, without authority inference.
fn native_owners<'a>(plan_key: &str, owners: &'a [Owner]) -> Vec<NativeOwner<'a>> {
    owners
        .iter()
        .map(|owner| NativeOwner {
            role_id: &owner.role_id,
            party_key: &owner.party_key,
            party_uuid: scoped_uuid(plan_key, "party", &owner.party_key).to_string(),
            rationale: &owner.rationale,
        })
        .collect()
}

/// Project all five matched source receipts without hidden reads or fabricated byte hashes.
fn source_resources(manifest: &WorkflowManifest) -> Result<BackMatter, ForgeError> {
    let declarations: [(&'static str, &ArtifactManifest); 5] = [
        ("assessment-results", &manifest.source.assessment_results),
        ("assessment-plan", &manifest.source.context.assessment_plan),
        ("system-security-plan", &manifest.source.context.ssp),
        ("profile", &manifest.source.context.profile),
        ("catalog", &manifest.source.context.catalog),
    ];
    let mut resources = Vec::with_capacity(5);
    for (kind, pin) in declarations {
        resources.push(Resource {
            uuid: scoped_uuid(&manifest.document.key, "source-receipt", kind).to_string(),
            title: kind,
            rlinks: vec![ResourceLink {
                href: relative_uri(&pin.artifact)?,
                media_type: "application/json",
                hashes: vec![Hash { algorithm: "SHA-256", value: pin.expected_sha256.clone() }],
            }],
        });
    }
    Ok(BackMatter { resources })
}

/// Encode bounded compact JSON into one native string property, escaping authored newlines.
///
/// Pretty outer artifacts/reports remain separate: `StringDatatype` properties must
/// not acquire serializer line delimiters. The complete declaration uses the same
/// pre-growth ten MiB writer and is never returned as a prefix on serialization error.
fn declaration(value: &impl Serialize) -> Result<String, ForgeError> {
    let mut writer = BoundedWriter { bytes: Vec::new() };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| error("complete declaration exceeds its bound or cannot serialize"))?;
    String::from_utf8(writer.bytes).map_err(|_| error("typed declaration is not UTF8"))
}

/// Construct one fixed known namespaced property without arbitrary native field injection.
fn property(name: &'static str, value: String) -> Property {
    Property { name, ns: NS, value }
}

/// Use the F07 length-prefixed protocol with new party/source receipt domains.
fn scoped_uuid(plan_key: &str, domain: &str, key: &str) -> Uuid {
    let mut bytes = Vec::new();
    for segment in [super::manifest::MANIFEST_SCHEMA_VERSION, plan_key, domain, key] {
        bytes.extend_from_slice(&(segment.len() as u64).to_be_bytes());
        bytes.extend_from_slice(segment.as_bytes());
    }
    Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, &bytes)
}

/// Encode a normalized confined artifact path as a URI reference without filesystem/network access.
fn relative_uri(path: &std::path::Path) -> Result<String, ForgeError> {
    let text = path.to_str().ok_or_else(|| error("source artifact path must be UTF8"))?;
    if path.is_absolute()
        || path.components().any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(error("source artifact URI must be normalized and relative"));
    }
    let mut output = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'.' | b'_' | b'~') {
            output.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(output, "%{byte:02X}").map_err(|_| error("source URI encoding failed"))?;
        }
    }
    Ok(output)
}

/// Validate complete native JSON against the existing offline pinned schema.
///
/// Sibling callers must first strict-parse and compare their actual original bytes
/// to the complete supported current projection; this validator grants no source authority.
pub(super) fn validate_native(bytes: &[u8]) -> Result<(), ForgeError> {
    let validator = VALIDATOR
        .get_or_init(|| {
            let value: serde_json::Value =
                serde_json::from_str(include_str!("../../schemas/oscal_poam_schema.json"))
                    .map_err(|_| "vendored POA&M schema is malformed".to_string())?;
            jsonschema::validator_for(&value)
                .map_err(|_| "vendored POA&M schema compilation failed".to_string())
        })
        .as_ref()
        .map_err(|_| error("offline POA&M schema is unavailable"))?;
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| error("generated native JSON is invalid"))?;
    if !validator.is_valid(&value) {
        return Err(error("generated POA&M fails the official pinned schema"));
    }
    Ok(())
}

/// Serialize the complete typed object through a pre-growth bounded writer with one final newline.
pub(super) fn json_line(value: &impl Serialize) -> Result<Vec<u8>, ForgeError> {
    let mut writer = BoundedWriter { bytes: Vec::new() };
    serde_json::to_writer_pretty(&mut writer, value)
        .map_err(|_| error("complete workflow output exceeds its bound or cannot serialize"))?;
    writer.write_all(b"\n").map_err(|_| error("complete workflow output exceeds its bound"))?;
    Ok(writer.bytes)
}

/// Writer that refuses the entire next append before growth beyond ten MiB.
struct BoundedWriter {
    /// Complete accepted prefix; never returned on a writer error.
    bytes: Vec<u8>,
}

impl std::io::Write for BoundedWriter {
    /// Admit bytes only when the complete append fits the remaining output budget.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_OUTPUT_BYTES.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("workflow output limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    /// Complete the in-memory writer interface without external IO.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Emit a fixed typed failure without authored prose, paths or arbitrary schema diagnostics.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.to_string())
}
