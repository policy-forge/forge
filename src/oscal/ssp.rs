//! OSCAL System Security Plan (SSP) builder.
//!
//! Produces an SSP JSON envelope with system-implementation containing
//! inventory-items, users, leveraged-authorizations, plus metadata,
//! back-matter, and implemented-requirements.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::error::ForgeError;
use crate::model::DocumentMetadata;
use crate::oscal::back_matter::{BackMatter, BackMatterResource, OscalLink};
use crate::oscal::catalog::{OscalCatalog, OscalGroup};
use crate::oscal::metadata::assemble_metadata;
use crate::oscal::parts::OscalProp;
use crate::uuid::{COMPONENT_NAMESPACE, generate_stable_id};
use uuid::Uuid;

// ─── Constants ──────────────────────────────────────────────────────────

/// OSCAL specification version for SSP documents — matches the shared `OSCAL_VERSION` constant.
pub const SSP_OSCAL_VERSION: &str = crate::oscal::metadata::OSCAL_VERSION;

/// Default SSP title when source document has no title.
pub const DEFAULT_SSP_TITLE: &str = "Untitled System Security Plan";

/// Seed namespace for deterministic SSP UUID generation.
const SSP_UUID_SEED: &str = "ssp";

/// Namespace for FORGE-only SSP template metadata that has no native OSCAL field.
const FORGE_SSP_TEMPLATE_NS: &str = "https://forge.policy-forge.github.io/ns/ssp-template";

// ─── Envelope ───────────────────────────────────────────────────────────

/// Top-level SSP JSON envelope producing `{"system-security-plan": {...}}`.
#[derive(Debug, Clone, Serialize)]
pub struct SystemSecurityPlanEnvelope {
    /// The root System Security Plan object (serialized under key `system-security-plan`).
    #[serde(rename = "system-security-plan")]
    pub system_security_plan: SystemSecurityPlan,
}

// ─── SSP Root ───────────────────────────────────────────────────────────

/// OSCAL System Security Plan root object.
#[derive(Debug, Clone, Serialize)]
pub struct SystemSecurityPlan {
    /// Document UUID (v5, deterministic from seed).
    pub uuid: String,

    /// Metadata block (title, version, oscal-version, etc.).
    pub metadata: SspMetadata,

    /// Legacy direct system identifier retained for callers; OSCAL 1.2.3 nests
    /// system identifiers under `system-characteristics`.
    #[serde(skip)]
    pub system_id: Option<SspSystemId>,

    /// Profile or catalog that defines the system's control baseline.
    #[serde(rename = "import-profile")]
    pub import_profile: ImportProfile,

    /// Required description of the system and its authorization boundary.
    #[serde(rename = "system-characteristics")]
    pub system_characteristics: SystemCharacteristics,

    /// System implementation details: components, users, leveraged auths.
    #[serde(rename = "system-implementation")]
    pub system_implementation: SystemImplementation,

    /// Control implementation statements.
    #[serde(default, rename = "control-implementation", skip_serializing_if = "Option::is_none")]
    pub control_implementation: Option<SspControlImplementation>,

    /// Back matter with resources and citations.
    #[serde(default, rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<BackMatter>,
}

// ─── Metadata ───────────────────────────────────────────────────────────

/// SSP-specific metadata block.
#[derive(Debug, Clone, Serialize)]
pub struct SspMetadata {
    /// Human-readable SSP title.
    pub title: String,

    /// ISO 8601 UTC timestamp of generation.
    #[serde(rename = "last-modified")]
    pub last_modified: String,

    /// Document version string.
    pub version: String,

    /// OSCAL spec version — always the shared v1.2.3 baseline.
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,

    /// Revision history entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<SspRevision>,
}

/// A single revision history entry.
#[derive(Debug, Clone, Serialize)]
pub struct SspRevision {
    /// Title/description of this revision.
    pub title: String,

    /// ISO 8601 timestamp of the revision.
    #[serde(rename = "last-modified")]
    pub last_modified: String,

    /// Version string for this revision.
    pub version: String,

    /// Human-readable description of what changed. OSCAL v1.2.3 represents
    /// this content as revision `remarks`.
    #[serde(default, rename = "remarks", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ─── System ID ──────────────────────────────────────────────────────────

/// System identifier with an optional identifying scheme (URI).
#[derive(Debug, Clone, Serialize)]
pub struct SspSystemId {
    /// Optional URN or URI identifying the scheme.
    #[serde(rename = "identifier-type", skip_serializing_if = "Option::is_none")]
    pub identifier_type: Option<String>,

    /// The actual system identifier value.
    pub id: String,
}

/// Reference to the profile or catalog used as the system control baseline.
#[derive(Debug, Clone, Serialize)]
pub struct ImportProfile {
    /// Resolvable URI reference to the source profile or catalog.
    pub href: String,
}

/// Required OSCAL SSP system characteristics.
#[derive(Debug, Clone, Serialize)]
pub struct SystemCharacteristics {
    /// One or more identifiers for the system.
    #[serde(rename = "system-ids")]
    pub system_ids: Vec<SspSystemId>,

    /// Full system name.
    #[serde(rename = "system-name")]
    pub system_name: String,

    /// Summary of the system purpose and scope.
    pub description: String,

    /// Information types stored, processed, or transmitted by the system.
    #[serde(rename = "system-information")]
    pub system_information: SystemInformation,

    /// Current operational state of the system.
    pub status: ComponentStatus,

    /// Narrative description of the authorization boundary.
    #[serde(rename = "authorization-boundary")]
    pub authorization_boundary: AuthorizationBoundary,
}

/// Information types handled by the system.
#[derive(Debug, Clone, Serialize)]
pub struct SystemInformation {
    /// At least one information type is required by OSCAL 1.2.3.
    #[serde(rename = "information-types")]
    pub information_types: Vec<InformationType>,
}

/// A category of information handled by the system.
#[derive(Debug, Clone, Serialize)]
pub struct InformationType {
    /// Human-readable information type name.
    pub title: String,

    /// Description of how the information is used.
    pub description: String,
}

/// Description of the system authorization boundary.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorizationBoundary {
    /// Narrative boundary description.
    pub description: String,
}

// ─── System Implementation ──────────────────────────────────────────────

/// Container for system components, users, and leveraged authorizations.
#[derive(Debug, Clone, Serialize)]
pub struct SystemImplementation {
    /// Inventory of system components.
    #[serde(default, rename = "components", skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<SspComponent>,

    /// Authorized users of the system.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<AuthorizedUser>,

    /// Leveraged authorizations (inherited from other systems).
    #[serde(default, rename = "leveraged-authorizations", skip_serializing_if = "Vec::is_empty")]
    pub leveraged_authorizations: Vec<LeveragedAuthorization>,
}

// ─── SspComponent (inventory item) ─────────────────────────────────────

/// A system component/inventory item in the SSP.
#[derive(Debug, Clone, Serialize)]
pub struct SspComponent {
    /// Unique identifier for this component.
    pub uuid: String,

    /// Component type (e.g., "software", "hardware", "service").
    #[serde(rename = "type")]
    pub component_type: String,

    /// Human-readable component title.
    pub title: String,

    /// Component description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Responsible parties for this component.
    #[serde(default, rename = "responsible-roles", skip_serializing_if = "Vec::is_empty")]
    pub responsible_roles: Vec<ResponsibleRole>,

    /// Status of the component.
    pub status: ComponentStatus,
}

/// Status of a system component.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatus {
    /// Operational state (e.g., "operational", "under-development", "disposition").
    pub state: String,
}

/// A responsible role assignment for a component.
#[derive(Debug, Clone, Serialize)]
pub struct ResponsibleRole {
    /// Role identifier (references a role-id).
    #[serde(rename = "role-id")]
    pub role_id: String,

    /// Display name or completion guidance for the responsible party. The
    /// legacy string cannot populate OSCAL's UUID-only `party-uuids`, so it is
    /// preserved as `remarks` rather than silently discarded.
    #[serde(default, rename = "remarks", skip_serializing_if = "Option::is_none")]
    pub responsible_party: Option<String>,
}

// ─── Component Definition (input) ──────────────────────────────────────

/// Input definition for a system component, used to generate inventory items.
///
/// Each `SspComponentInput` maps to one `SspComponent` in the SSP's
/// system-implementation section.
#[derive(Debug, Clone)]
pub struct SspComponentInput {
    /// Human-readable title of the component.
    pub title: String,

    /// Description of the component's purpose and scope.
    pub description: String,

    /// OSCAL component type (e.g., "software", "hardware", "service", "network").
    pub component_type: String,
}

/// Generate inventory-items (`SspComponent` entries) from component definitions.
///
/// For each `SspComponentInput`, produces an `SspComponent` with:
/// - A deterministic `UUIDv5` derived from the component title using `COMPONENT_NAMESPACE`
/// - Description from the definition
/// - No fabricated responsible-role reference; callers add one with a real role ID
/// - Operational status set to "operational" by default
///
/// # Arguments
///
/// * `definitions` — Slice of component definitions to convert.
///
/// # Returns
///
/// A `Vec<SspComponent>` ready to populate `SystemImplementation::components`.
///
/// # Examples
///
/// ```
/// use forge::oscal::ssp::{SspComponentInput, generate_inventory_items};
///
/// let defs = vec![
///     SspComponentInput {
///         title: "Web Application Firewall".to_string(),
///         description: "Filters incoming HTTP traffic".to_string(),
///         component_type: "software".to_string(),
///     },
/// ];
/// let items = generate_inventory_items(&defs).unwrap();
/// assert_eq!(items.len(), 1);
/// assert_eq!(items[0].title, "Web Application Firewall");
/// ```
///
/// # Errors
///
/// Returns [`ForgeError::SspBuild`] when normalized component titles duplicate
/// and would otherwise derive the same deterministic component UUID (F0624).
pub fn generate_inventory_items(
    definitions: &[SspComponentInput],
) -> Result<Vec<SspComponent>, ForgeError> {
    // Component UUIDs derive from the title alone, so duplicate titles would
    // silently mint identical UUIDs and merge distinct components (F0624).
    let mut seen_titles = std::collections::HashSet::new();
    for def in definitions {
        if !seen_titles.insert(def.title.trim().to_lowercase()) {
            return Err(ForgeError::SspBuild(format!(
                "duplicate component title {:?}: title-derived UUIDv5 would collide",
                def.title
            )));
        }
    }
    Ok(definitions
        .iter()
        .map(|def| {
            let uuid = Uuid::new_v5(&COMPONENT_NAMESPACE, def.title.as_bytes()).to_string();
            SspComponent {
                uuid,
                component_type: def.component_type.clone(),
                title: def.title.clone(),
                description: Some(def.description.clone()),
                responsible_roles: vec![],
                status: ComponentStatus { state: "operational".to_string() },
            }
        })
        .collect())
}

// ─── Authorized Users ───────────────────────────────────────────────────

/// An authorized user of the system.
///
/// OSCAL v1.2.3 user object within system-implementation.
/// Fields marked with `<!-- TODO(...) -->` need to be filled with real data.
#[derive(Debug, Clone)]
pub struct AuthorizedUser {
    /// Unique identifier for this user entry.
    pub uuid: String,

    /// Full name or display title for the user.
    /// <!-- TODO(title): set the actual user display name -->
    pub title: String,

    /// Short identifier (e.g., username, employee ID).
    /// <!-- TODO(short-name): set the actual username or employee ID -->
    pub short_name: Option<String>,

    /// Description of the user's role and access scope.
    /// <!-- TODO(description): describe the user's purpose and access scope -->
    pub description: Option<String>,

    /// Role identifiers assigned to this user.
    /// References role-ids defined in the system or inherited from the SSP.
    /// <!-- TODO(role-ids): assign concrete role-ids matching the system's role catalog -->
    pub role_ids: Vec<String>,

    /// Date the user was authorized (ISO 8601).
    /// <!-- TODO(authorized-date): set the actual authorization date -->
    pub authorized_date: Option<String>,

    /// Legacy user account status retained for callers. OSCAL 1.2.3 system
    /// users do not define a status field.
    pub status: UserStatus,
}

impl Serialize for AuthorizedUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut props = Vec::new();
        if let Some(authorized_date) = &self.authorized_date {
            props.push(template_prop("authorized-date", authorized_date));
        }
        props.push(template_prop("account-status", &self.status.state));
        if let Some(reason) = &self.status.reason {
            props.push(template_prop("account-status-reason", reason));
        }

        if self.role_ids.is_empty() {
            props.push(template_prop(
                "role-ids",
                "TODO: Define metadata roles and assign their role IDs.",
            ));
        }

        let field_count = 2
            + usize::from(self.short_name.is_some())
            + usize::from(self.description.is_some())
            + usize::from(!self.role_ids.is_empty())
            + usize::from(!props.is_empty());
        let mut state = serializer.serialize_struct("AuthorizedUser", field_count)?;
        state.serialize_field("uuid", &self.uuid)?;
        state.serialize_field("title", &self.title)?;
        if let Some(short_name) = &self.short_name {
            state.serialize_field("short-name", short_name)?;
        }
        if let Some(description) = &self.description {
            state.serialize_field("description", description)?;
        }
        if !self.role_ids.is_empty() {
            state.serialize_field("role-ids", &self.role_ids)?;
        }
        if !props.is_empty() {
            state.serialize_field("props", &props)?;
        }
        state.end()
    }
}

fn template_prop(name: &str, value: &str) -> OscalProp {
    OscalProp {
        name: name.to_string(),
        ns: Some(FORGE_SSP_TEMPLATE_NS.to_string()),
        value: value.to_string(),
    }
}

/// Status of a user account.
#[derive(Debug, Clone, Serialize)]
pub struct UserStatus {
    /// Account state: "active", "inactive", "pending", "disabled".
    pub state: String,

    /// Optional reason for the current state.
    /// <!-- TODO(reason): explain why the account is in this state if not active -->
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ─── Leveraged Authorizations ───────────────────────────────────────────

/// A leveraged (inherited) authorization from another system.
///
/// OSCAL v1.2.3 leveraged-authorization object within system-implementation.
/// Represents an authority to inherit controls from a shared service/system.
#[derive(Debug, Clone)]
pub struct LeveragedAuthorization {
    /// Unique identifier for this leveraged authorization entry.
    pub uuid: String,

    /// Title of the leveraged system or service.
    /// <!-- TODO(title): set the leveraged system name (e.g., "AWS GovCloud") -->
    pub title: String,

    /// Hyperlink reference to the leveraged system's SSP.
    /// <!-- TODO(href): set the URL or path to the leveraged system's SSP -->
    pub href: Option<String>,

    /// Description of what is inherited.
    /// <!-- TODO(description): describe the inherited controls or services -->
    pub description: Option<String>,

    /// UUID of the party that manages the leveraged system.
    pub party_uuid: String,

    /// Date the leveraged system received authorization.
    pub date_authorized: String,
}

impl Serialize for LeveragedAuthorization {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 4;
        field_count += usize::from(self.href.is_some());
        field_count += usize::from(self.description.is_some());

        let mut state = serializer.serialize_struct("LeveragedAuthorization", field_count)?;
        state.serialize_field("uuid", &self.uuid)?;
        state.serialize_field("title", &self.title)?;
        if let Some(href) = &self.href {
            let links = [OscalLink {
                href: href.clone(),
                rel: "source".to_string(),
                text: Some("Leveraged system SSP".to_string()),
            }];
            state.serialize_field("links", &links)?;
        }
        state.serialize_field("party-uuid", &self.party_uuid)?;
        state.serialize_field("date-authorized", &self.date_authorized)?;
        if let Some(description) = &self.description {
            state.serialize_field("remarks", description)?;
        }
        state.end()
    }
}

// ─── Control Implementation ─────────────────────────────────────────────

/// Implementation status of a control for a specific component.
#[derive(Debug, Clone, Serialize)]
pub struct ImplementationStatus {
    /// One of: "planned", "implemented", "partial", "not-applicable".
    pub state: String,
}

/// A by-component entry linking an implemented-requirement to a component.
#[derive(Debug, Clone, Serialize)]
pub struct ByComponent {
    /// Stable UUID for this component-specific implementation statement.
    pub uuid: String,

    /// UUID of the component this implementation applies to.
    #[serde(rename = "component-uuid")]
    pub component_uuid: String,

    /// Implementation status for this component.
    #[serde(rename = "implementation-status")]
    pub implementation_status: ImplementationStatus,

    /// Description of how the control is implemented for this component.
    pub description: String,
}

/// An implemented requirement within an SSP control-implementation.
///
/// Each maps to one catalog control and carries by-component entries
/// linking to the system's components.
#[derive(Debug, Clone, Serialize)]
pub struct SspImplementedRequirement {
    /// Deterministic UUID v5 derived from control-id.
    pub uuid: String,

    /// Control ID matching the Catalog builder's scheme (e.g., `"POL-AC-001"`).
    #[serde(rename = "control-id")]
    pub control_id: String,

    /// Implementation narrative (placeholder with TODO markers). OSCAL v1.2.3
    /// represents this content as implemented-requirement `remarks`.
    #[serde(rename = "remarks")]
    pub description: String,

    /// Trace properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,

    /// Source links for traceability.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<OscalLink>,

    /// Component-level implementation entries for this control.
    #[serde(default, rename = "by-components", skip_serializing_if = "Vec::is_empty")]
    pub by_components: Vec<ByComponent>,
}

/// Container for implemented requirements within an SSP.
#[derive(Debug, Clone, Serialize)]
pub struct SspControlImplementation {
    /// Description of the control implementation approach.
    pub description: String,

    /// Implemented requirements (one per control).
    #[serde(rename = "implemented-requirements")]
    pub implemented_requirements: Vec<SspImplementedRequirement>,
}

// ─── Builder ────────────────────────────────────────────────────────────

/// Build a placeholder System Security Plan with skeleton sections.
///
/// Produces an SSP with:
/// - Deterministic UUID (v5 from policy title)
/// - Metadata with TODO markers for revision history
/// - Empty system-id placeholder
/// - System-implementation with placeholder users and components
/// - Back-matter with placeholder resources
///
/// # Arguments
///
/// * `policy_title` — Title of the source policy document.
/// * `policy_version` — Version of the source policy document.
///
/// # Errors
///
/// * `ForgeError::SspBuild` — if metadata assembly fails
pub fn build_ssp(
    policy_title: &str,
    policy_version: &str,
) -> Result<SystemSecurityPlanEnvelope, ForgeError> {
    let title = if policy_title.trim().is_empty() { DEFAULT_SSP_TITLE } else { policy_title };

    // Assemble metadata from shared function
    let doc_meta = DocumentMetadata {
        title: format!("System Security Plan for {title}"),
        version: policy_version.to_string(),
        ..Default::default()
    };
    let real_metadata =
        assemble_metadata(&doc_meta, None).map_err(|e| ForgeError::SspBuild(e.to_string()))?;

    // Deterministic UUID v5 from policy title
    let seed = format!("{SSP_UUID_SEED}|{title}");
    let uuid = generate_stable_id(&seed).to_string();

    // Build system-implementation with placeholder users
    let system_implementation = build_system_implementation();

    // Build back-matter with placeholder resources
    let back_matter = build_ssp_back_matter();

    let envelope = SystemSecurityPlanEnvelope {
        system_security_plan: SystemSecurityPlan {
            uuid,
            metadata: SspMetadata {
                title: real_metadata.title,
                last_modified: real_metadata.last_modified.to_rfc3339(),
                version: real_metadata.version,
                oscal_version: SSP_OSCAL_VERSION.to_string(),
                revisions: vec![SspRevision {
                    title: "Initial SSP generation".to_string(),
                    last_modified: real_metadata.last_modified.to_rfc3339(),
                    version: "1.0.0".to_string(),
                    description: Some("<!-- TODO(description): describe initial SSP scope and system boundary -->".to_string()),
                }],
            },
            system_id: None,
            import_profile: ImportProfile { href: "TODO-profile.json".to_string() },
            system_characteristics: build_system_characteristics(title),
            system_implementation,
            control_implementation: None,
            back_matter: Some(back_matter),
        },
    };

    tracing::info!(ssp_uuid = %envelope.system_security_plan.uuid, title, "SSP generated with placeholder users");

    Ok(envelope)
}

/// Build a full SSP skeleton with control-implementation and inventory components.
///
/// Extends [`build_ssp`] by layering control-implementation (derived from a Catalog)
/// and system-implementation components (derived from component definitions).
///
/// # Arguments
///
/// * `policy_title` — Title of the source policy document.
/// * `policy_version` — Version of the source policy document.
/// * `catalog` — OSCAL Catalog containing controls to derive implemented-requirements from.
/// * `component_defs` — Component definitions to generate system-implementation inventory from.
/// * `source_profile` — Path to the source profile for SSP metadata.
///
/// # Returns
///
/// A `SystemSecurityPlanEnvelope` with populated control-implementation and inventory items.
///
/// # Errors
///
/// * `ForgeError::SspBuild` — if the base SSP or control-implementation construction fails.
pub fn build_ssp_skeleton(
    policy_title: &str,
    policy_version: &str,
    catalog: &OscalCatalog,
    component_defs: &[SspComponentInput],
    source_profile: &str,
) -> Result<SystemSecurityPlanEnvelope, ForgeError> {
    // Build the base SSP envelope
    let mut envelope = build_ssp(policy_title, policy_version)?;

    // Collect all control IDs from the catalog
    let control_ids = collect_catalog_control_ids(catalog);

    // Generate inventory items from component definitions
    let inventory_items = generate_inventory_items(component_defs)?;

    // Collect component UUIDs for by-component linking
    let component_uuids: Vec<&str> = inventory_items.iter().map(|c| c.uuid.as_str()).collect();

    // Build implemented-requirements: one per catalog control
    let implemented_requirements = build_control_impl_reqs(&control_ids, &component_uuids);

    // Build control-implementation section
    let ssp = &mut envelope.system_security_plan;
    ssp.import_profile.href = if source_profile.trim().is_empty() {
        "TODO-profile.json".to_string()
    } else {
        source_profile.to_string()
    };
    ssp.control_implementation = Some(SspControlImplementation {
        description: format!("Control implementation statements derived from {policy_title}."),
        implemented_requirements,
    });

    // Replace empty components with inventory items
    ssp.system_implementation.components = inventory_items;

    Ok(envelope)
}

/// Build the required system-characteristics skeleton with schema-valid TODO
/// values that users can replace as they complete the SSP.
fn build_system_characteristics(system_name: &str) -> SystemCharacteristics {
    SystemCharacteristics {
        system_ids: vec![todo_system_id()],
        system_name: system_name.to_string(),
        description: "TODO: Describe the system purpose, environment, and scope.".to_string(),
        system_information: SystemInformation {
            information_types: vec![InformationType {
                title: "TODO information type".to_string(),
                description:
                    "TODO: Describe how this information is stored, processed, or transmitted."
                        .to_string(),
            }],
        },
        status: ComponentStatus { state: "under-development".to_string() },
        authorization_boundary: AuthorizationBoundary {
            description: "TODO: Describe the system authorization boundary.".to_string(),
        },
    }
}

fn todo_system_id() -> SspSystemId {
    SspSystemId {
        identifier_type: Some("https://example.com/system-identifiers".to_string()),
        id: "TODO-system-id".to_string(),
    }
}

/// Recursively collect all control IDs from a catalog (top-level + groups).
fn collect_catalog_control_ids(catalog: &OscalCatalog) -> Vec<String> {
    let mut ids = Vec::new();
    ids.extend(catalog.controls.iter().map(|c| c.id.clone()));
    for group in &catalog.groups {
        collect_group_control_ids(group, &mut ids);
    }
    ids
}

/// Recursively collect control IDs from a group (nested groups + controls).
fn collect_group_control_ids(group: &OscalGroup, ids: &mut Vec<String>) {
    ids.extend(group.controls.iter().map(|c| c.id.clone()));
    for subgroup in &group.groups {
        collect_group_control_ids(subgroup, ids);
    }
}

/// Build implemented-requirements for a set of control IDs.
///
/// Each control gets an `SspImplementedRequirement` with:
/// - Deterministic UUID v5 derived from the control-id
/// - Implementation-status placeholder ("planned")
/// - By-component entries linking all component UUIDs
fn build_control_impl_reqs(
    control_ids: &[String],
    component_uuids: &[&str],
) -> Vec<SspImplementedRequirement> {
    control_ids
        .iter()
        .map(|control_id| {
            let uuid = generate_stable_id(&format!("ssp-impl-req|{control_id}")).to_string();
            let by_components = if component_uuids.is_empty() {
                vec![]
            } else {
                component_uuids
                    .iter()
                    .map(|comp_uuid| ByComponent {
                        uuid: generate_stable_id(&format!(
                            "ssp-by-component|{control_id}|{comp_uuid}"
                        ))
                        .to_string(),
                        component_uuid: (*comp_uuid).to_string(),
                        implementation_status: ImplementationStatus {
                            state: "planned".to_string(),
                        },
                        description: format!(
                            "<!-- TODO(by-component): describe how control {control_id} is implemented for this component -->"
                        ),
                    })
                    .collect()
            };
            SspImplementedRequirement {
                uuid,
                control_id: control_id.clone(),
                description: format!(
                    "<!-- TODO(description): implementation narrative for control {control_id} -->"
                ),
                props: vec![],
                links: vec![],
                by_components,
            }
        })
        .collect()
}

/// Build the system-implementation section with placeholder users, components,
/// and leveraged authorizations.
fn build_system_implementation() -> SystemImplementation {
    SystemImplementation {
        components: Vec::new(), // Populated by inventory-items task
        users: build_placeholder_users(),
        // A leveraged authorization is optional. Do not fabricate a real-looking
        // authorization date or a dangling party reference merely to provide a template.
        leveraged_authorizations: vec![],
    }
}

/// Build placeholder authorized-user entries with role-ids.
///
/// These are skeleton entries demonstrating the OSCAL user structure.
/// Each has TODO markers for fields that need real data.
fn build_placeholder_users() -> Vec<AuthorizedUser> {
    vec![
        AuthorizedUser {
            uuid: generate_stable_id("user|system-admin").to_string(),
            title: "<!-- TODO(title): set system administrator name -->".to_string(),
            short_name: Some("<!-- TODO(short-name): set admin username, e.g., 'sysadmin' -->".to_string()),
            description: Some(
                "<!-- TODO(description): describe system administrator role and access scope -->"
                    .to_string(),
            ),
            role_ids: vec![],
            authorized_date: Some(
                "<!-- TODO(authorized-date): set authorization date in ISO 8601 format -->"
                    .to_string(),
            ),
            status: UserStatus {
                state: "active".to_string(),
                reason: None,
            },
        },
        AuthorizedUser {
            uuid: generate_stable_id("user|regular-user").to_string(),
            title: "<!-- TODO(title): set regular user display name -->".to_string(),
            short_name: Some(
                "<!-- TODO(short-name): set regular username, e.g., 'analyst-01' -->".to_string(),
            ),
            description: Some(
                "<!-- TODO(description): describe regular user role and access scope -->"
                    .to_string(),
            ),
            role_ids: vec![],
            authorized_date: Some(
                "<!-- TODO(authorized-date): set authorization date in ISO 8601 format -->"
                    .to_string(),
            ),
            status: UserStatus {
                state: "active".to_string(),
                reason: None,
            },
        },
        AuthorizedUser {
            uuid: generate_stable_id("user|service-account").to_string(),
            title: "<!-- TODO(title): set service account name -->".to_string(),
            short_name: Some(
                "<!-- TODO(short-name): set service account identifier, e.g., 'svc-ingest' -->"
                    .to_string(),
            ),
            description: Some(
                "<!-- TODO(description): describe service account purpose, e.g., 'automated data ingestion' -->"
                    .to_string(),
            ),
            role_ids: vec![],
            authorized_date: Some(
                "<!-- TODO(authorized-date): set authorization date in ISO 8601 format -->"
                    .to_string(),
            ),
            status: UserStatus {
                state: "active".to_string(),
                reason: Some(
                    "<!-- TODO(reason): e.g., 'automated system process, no interactive login' -->"
                        .to_string(),
                ),
            },
        },
    ]
}

/// Build placeholder back-matter for the SSP.
fn build_ssp_back_matter() -> BackMatter {
    BackMatter {
        resources: vec![
            BackMatterResource {
                uuid: generate_stable_id("resource|ssp-template"),
                title: "<!-- TODO(title): set the SSP template or source document reference -->"
                    .to_string(),
                description: Some(
                    "<!-- TODO(description): describe the source policy or template used -->"
                        .to_string(),
                ),
                citation: None,
                rlinks: vec![],
                props: vec![],
            },
            BackMatterResource {
                uuid: generate_stable_id("resource|system-diagram"),
                title: "<!-- TODO(title): set system architecture diagram reference -->"
                    .to_string(),
                description: Some(
                    "<!-- TODO(description): describe the system boundary diagram -->".to_string(),
                ),
                citation: None,
                rlinks: vec![],
                props: vec![],
            },
        ],
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssp_root_key_is_system_security_plan() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"system-security-plan\""), "Root key must be system-security-plan");
    }

    #[test]
    fn ssp_metadata_fields_present() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let ssp = &envelope.system_security_plan;
        assert!(ssp.metadata.title.contains("Test Policy"));
        assert_eq!(ssp.metadata.oscal_version, crate::oscal::metadata::OSCAL_VERSION);
        assert!(!ssp.metadata.last_modified.is_empty());
        assert_eq!(ssp.metadata.version, "1.0.0");
    }

    #[test]
    fn ssp_does_not_populate_legacy_system_id() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();

        assert!(envelope.system_security_plan.system_id.is_none());
        assert_eq!(envelope.system_security_plan.system_characteristics.system_ids.len(), 1);
    }

    #[test]
    fn ssp_has_revisions_with_todo_markers() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let revisions = &envelope.system_security_plan.metadata.revisions;
        assert_eq!(revisions.len(), 1);
        assert_eq!(revisions[0].title, "Initial SSP generation");
        assert!(revisions[0].description.as_deref().unwrap().contains("TODO"));
        let json = serde_json::to_value(&envelope).unwrap();
        assert!(
            json["system-security-plan"]["metadata"]["revisions"][0]["remarks"]
                .as_str()
                .is_some_and(|remarks| remarks.contains("TODO"))
        );
    }

    #[test]
    fn ssp_system_implementation_has_users() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let users = &envelope.system_security_plan.system_implementation.users;
        assert!(!users.is_empty(), "SSP must have placeholder users");
        assert_eq!(users.len(), 3, "Expected 3 placeholder user entries");
    }

    #[test]
    fn ssp_users_do_not_reference_undefined_roles() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let users = &envelope.system_security_plan.system_implementation.users;
        for user in users {
            assert!(user.role_ids.is_empty());
        }

        let json = serde_json::to_value(&envelope).unwrap();
        for user in
            json["system-security-plan"]["system-implementation"]["users"].as_array().unwrap()
        {
            assert!(user.get("role-ids").is_none());
            assert!(user["props"].as_array().unwrap().iter().any(|prop| {
                prop["name"] == "role-ids"
                    && prop["value"].as_str().is_some_and(|value| value.contains("TODO"))
            }));
        }
    }

    #[test]
    fn authorized_user_serializes_every_emitted_field() {
        let user = AuthorizedUser {
            uuid: "user-uuid".to_string(),
            title: "Named user".to_string(),
            short_name: Some("named".to_string()),
            description: Some("A test user".to_string()),
            role_ids: vec!["system-owner".to_string()],
            authorized_date: Some("2026-08-26".to_string()),
            status: UserStatus {
                state: "active".to_string(),
                reason: Some("approved".to_string()),
            },
        };

        let json = serde_json::to_value(user).unwrap();

        assert_eq!(json["role-ids"][0].as_str(), Some("system-owner"));
        assert_eq!(json["props"].as_array().map_or(0, Vec::len), 3);
    }

    #[test]
    fn ssp_users_have_unique_uuids() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let users = &envelope.system_security_plan.system_implementation.users;
        let mut uuids: Vec<&str> = users.iter().map(|u| u.uuid.as_str()).collect();
        uuids.sort_unstable();
        uuids.dedup();
        assert_eq!(uuids.len(), users.len(), "All user UUIDs must be unique");
    }

    #[test]
    fn ssp_users_have_valid_status() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let users = &envelope.system_security_plan.system_implementation.users;
        for user in users {
            assert_eq!(user.status.state, "active");
        }
    }

    #[test]
    fn ssp_users_contain_todo_markers_in_fields() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let users = &envelope.system_security_plan.system_implementation.users;
        for user in users {
            assert!(user.title.contains("TODO"), "title should have TODO: {}", user.title);
            if let Some(ref sn) = user.short_name {
                assert!(sn.contains("TODO"), "short_name should have TODO: {sn}");
            }
            if let Some(ref desc) = user.description {
                assert!(desc.contains("TODO"), "description should have TODO: {desc}");
            }
        }

        let json = serde_json::to_value(&envelope).unwrap();
        let props = json["system-security-plan"]["system-implementation"]["users"][0]["props"]
            .as_array()
            .unwrap();
        assert!(props.iter().any(|prop| {
            prop["name"] == "authorized-date"
                && prop["value"].as_str().is_some_and(|value| value.contains("TODO"))
        }));
        assert!(props.iter().any(|prop| prop["name"] == "account-status"));
    }

    #[test]
    fn ssp_does_not_fabricate_leveraged_authorization() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let leveraged =
            &envelope.system_security_plan.system_implementation.leveraged_authorizations;
        assert!(leveraged.is_empty());

        let json = serde_json::to_string(&envelope).unwrap();
        assert!(!json.contains("leveraged-authorizations"));
        assert!(!json.contains("1970-01-01"));
        assert!(!json.contains("leveraged-auth-placeholder"));
        assert!(json.contains("TODO-profile.json"));
    }

    #[test]
    fn leveraged_authorization_legacy_fields_map_to_oscal_fields() {
        let authorization = LeveragedAuthorization {
            uuid: "b35fef25-14dc-4f91-b4fa-997c0f4d2a71".to_string(),
            title: "Shared service".to_string(),
            href: Some("shared-service-ssp.json".to_string()),
            description: Some("Inherited physical controls".to_string()),
            party_uuid: "13ac0950-94d9-4b81-8235-9d93bfeff0fc".to_string(),
            date_authorized: "2026-08-24".to_string(),
        };
        let json = serde_json::to_value(authorization).unwrap();
        assert_eq!(json["links"][0]["href"], "shared-service-ssp.json");
        assert_eq!(json["remarks"], "Inherited physical controls");
        assert_eq!(json["party-uuid"], "13ac0950-94d9-4b81-8235-9d93bfeff0fc");
    }

    #[test]
    fn ssp_has_back_matter() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let bm = envelope.system_security_plan.back_matter.as_ref().unwrap();
        assert!(!bm.resources.is_empty(), "Back-matter must have placeholder resources");
    }

    #[test]
    fn ssp_uuid_is_deterministic() {
        let a = build_ssp("Test Policy", "1.0.0").unwrap();
        let b = build_ssp("Test Policy", "1.0.0").unwrap();
        assert_eq!(
            a.system_security_plan.uuid, b.system_security_plan.uuid,
            "Same inputs must produce identical UUIDs"
        );
    }

    #[test]
    fn ssp_different_titles_different_uuids() {
        let a = build_ssp("Policy A", "1.0.0").unwrap();
        let b = build_ssp("Policy B", "1.0.0").unwrap();
        assert_ne!(
            a.system_security_plan.uuid, b.system_security_plan.uuid,
            "Different titles must produce different UUIDs"
        );
    }

    #[test]
    fn ssp_default_title_when_empty() {
        let envelope = build_ssp("", "1.0.0").unwrap();
        assert!(envelope.system_security_plan.metadata.title.contains(DEFAULT_SSP_TITLE));
    }

    #[test]
    fn ssp_json_is_parseable() {
        let envelope = build_ssp("Test Policy", "1.0.0").unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("system-security-plan").is_some());
    }

    // === Inventory Items Generation ===

    use super::{SspComponentInput, generate_inventory_items};

    #[test]
    fn generate_inventory_items_produces_components() {
        let defs = vec![
            SspComponentInput {
                title: "Web Application Firewall".to_string(),
                description: "Filters incoming HTTP traffic".to_string(),
                component_type: "software".to_string(),
            },
            SspComponentInput {
                title: "Database Server".to_string(),
                description: "Primary PostgreSQL instance".to_string(),
                component_type: "software".to_string(),
            },
        ];
        let items = generate_inventory_items(&defs).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Web Application Firewall");
        assert_eq!(items[1].title, "Database Server");
    }

    #[test]
    fn generate_inventory_items_deterministic_uuids() {
        let defs = vec![SspComponentInput {
            title: "Test Component".to_string(),
            description: "A test".to_string(),
            component_type: "software".to_string(),
        }];
        let items_a = generate_inventory_items(&defs).unwrap();
        let items_b = generate_inventory_items(&defs).unwrap();
        assert_eq!(
            items_a[0].uuid, items_b[0].uuid,
            "Same component title must produce identical UUID"
        );
    }

    #[test]
    fn generate_inventory_items_unique_uuids_per_component() {
        let defs = vec![
            SspComponentInput {
                title: "Component A".to_string(),
                description: "A".to_string(),
                component_type: "software".to_string(),
            },
            SspComponentInput {
                title: "Component B".to_string(),
                description: "B".to_string(),
                component_type: "hardware".to_string(),
            },
        ];
        let items = generate_inventory_items(&defs).unwrap();
        assert_ne!(items[0].uuid, items[1].uuid, "Different titles must produce different UUIDs");
    }

    #[test]
    fn generate_inventory_items_do_not_fabricate_responsible_roles() {
        let defs = vec![SspComponentInput {
            title: "Firewall".to_string(),
            description: "Network security".to_string(),
            component_type: "network".to_string(),
        }];
        let items = generate_inventory_items(&defs).unwrap();
        assert!(items[0].responsible_roles.is_empty());
        let json = serde_json::to_value(&items).unwrap();
        assert!(json[0].get("responsible-roles").is_none());
    }

    #[test]
    fn responsible_party_legacy_field_serializes_as_remarks() {
        let role = ResponsibleRole {
            role_id: "system-owner".to_string(),
            responsible_party: Some("TODO: assign a real party UUID".to_string()),
        };
        let json = serde_json::to_value(role).unwrap();
        assert_eq!(json["remarks"], "TODO: assign a real party UUID");
    }

    #[test]
    fn generate_inventory_items_empty_input() {
        let items = generate_inventory_items(&[]).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn generate_inventory_items_serialization_roundtrip() {
        let defs = vec![SspComponentInput {
            title: "Load Balancer".to_string(),
            description: "Distributes traffic across instances".to_string(),
            component_type: "service".to_string(),
        }];
        let items = generate_inventory_items(&defs).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["title"], "Load Balancer");
        assert_eq!(parsed[0]["type"], "service");
        assert_eq!(parsed[0]["status"]["state"], "operational");
        assert!(parsed[0]["description"].as_str().unwrap().contains("Distributes"));
    }
}
