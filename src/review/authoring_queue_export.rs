//! Genuine Init-only Queue export retaining the complete native plan and actual U owner.
//! Asserted declarations remain private data; generated output is never an input Queue.
use super::authoring_capture::CurrentAuthoringPlanClosure;
use super::capture::authoring::AuthoringOperation;
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase};
use super::decode_v3::DecodedV3;
use super::hash_v3::{self, NativeProvenance};
use super::wire_v3::{
    ADAPTER, ContextSnapshot, Disposition, DomainV3, IDENTITY_DISCLAIMER, QueueDocumentV3,
    RequestedAction, ReviewItemV3, Sensitivity, SourceKindV3, SourcePinV3,
};
use crate::workspace::preparation::WorkControl;
use std::mem::size_of;

/// Complete derived Queue with its genuine native owner and actual captured policy allocation.
pub(crate) struct PreparedAuthoringQueue<'owner> {
    /// The receiver-issued whole native/current saved-plan owner, never detached DTO proof.
    closure: &'owner CurrentAuthoringPlanClosure,
    /// Complete internally selected Init policy original, not copied or caller supplied.
    policy_raw: &'owner [u8],
    /// Fully admitted immutable completed Queue data.
    document: QueueDocumentV3,
}
/// Actual finite output and complete expected Queue stay coupled to the genuine owner.
pub(crate) struct EncodedAuthoringQueue<'owner> {
    /// Genuine original native and physical input lifetime.
    prepared: PreparedAuthoringQueue<'owner>,
    /// Complete finite output including its one LF; never registered as input.
    output: Vec<u8>,
}
impl<'owner> PreparedAuthoringQueue<'owner> {
    /// Borrow complete plain declarations for genuine fixture assertions only.
    #[cfg(test)]
    pub(crate) fn document(&self) -> &QueueDocumentV3 {
        &self.document
    }
    /// Consume native-bound preparation into strict complete finite output on the same work owner.
    pub(crate) fn encode(
        self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<EncodedAuthoringQueue<'owner>, ContractError> {
        phase(ledger, control, |ledger, control| {
            self.verify_inputs(ledger, control)?;
            let output = super::encode_v3::queue(&self.document, ledger, control)?;
            let decoded = super::decode_v3::decode_queue(&output, ledger, control)?;
            self.bind_output(&decoded, ledger, control)?;
            ledger.derived(size_of::<EncodedAuthoringQueue<'owner>>())?;
            Ok(EncodedAuthoringQueue { prepared: self, output })
        })
    }
    /// Compare every actual strict output field to the privately native-derived complete Queue.
    /// No plain decoder, pin list or caller digest can manufacture this expected result.
    pub(crate) fn bind_output(
        &self,
        actual: &DecodedV3<'_, QueueDocumentV3>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            super::encode_v3::compare_queues(&self.document, actual.document(), ledger, control)?;
            let declared = &actual.document().source_pins;
            let genuine = self.closure.source_pins();
            ledger.visits(2)?;
            if declared.len() != genuine.len() {
                return Err(ContractError::Binding);
            }
            for (expected, observed) in genuine.iter().zip(declared) {
                checkpoint(ledger, control)?;
                pin_work(expected, ledger)?;
                pin_work(observed, ledger)?;
                if expected != observed {
                    return Err(ContractError::Binding);
                }
            }
            self.verify_inputs(ledger, control)
        })
    }
    /// Check actual full-extent policy correlation and every real native/review input.
    fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            let actual = init_original(self.closure, ledger)?;
            ledger.visits(2)?;
            ledger.bytes(2 * size_of::<usize>())?;
            if actual.len() != self.policy_raw.len() || actual.as_ptr() != self.policy_raw.as_ptr()
            {
                return Err(ContractError::Binding);
            }
            self.closure.verify_inputs(ledger, control)
        })
    }
}
impl EncodedAuthoringQueue<'_> {
    /// Borrow actual complete finite bytes only while the genuine owner remains held.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.output
    }
    /// Original-ledger whole U fence for Root's guarded pre-rename publication callback.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        self.prepared.verify_inputs(ledger, control)
    }
}

/// Internally issued Init policy only; current-operation Queue and Response inputs are forbidden.
fn init_original<'owner>(
    closure: &'owner CurrentAuthoringPlanClosure,
    ledger: &mut ContractLedger,
) -> Result<&'owner [u8], ContractError> {
    ledger.visits(4)?;
    let originals = closure.originals()?;
    if originals.operation() != AuthoringOperation::Init
        || originals.queue_raw().is_some()
        || originals.responses().len() != 0
    {
        return Err(ContractError::Binding);
    }
    originals.policy_raw().ok_or(ContractError::Binding)
}
/// One complete string copy, with payload/representation/work admission before allocation.
fn copy(value: &str, ledger: &mut ContractLedger) -> Result<String, ContractError> {
    let extent = value.len().checked_add(size_of::<String>()).ok_or_else(|| ledger.capacity())?;
    ledger.derived(extent)?;
    ledger.bytes(value.len())?;
    ledger.visits(1)?;
    Ok(value.to_owned())
}
/// Exact null/present copy, retaining present-empty rather than defaulting it away.
fn copy_optional(
    value: Option<&str>,
    ledger: &mut ContractLedger,
) -> Result<Option<String>, ContractError> {
    ledger.visits(1)?;
    value.map(|value| copy(value, ledger)).transpose()
}
/// Whole actual list storage is admitted once before any vector growth or element production.
fn list<T>(count: usize, ledger: &mut ContractLedger) -> Result<Vec<T>, ContractError> {
    let extent = count
        .checked_mul(size_of::<T>())
        .and_then(|n| n.checked_add(size_of::<Vec<T>>()))
        .ok_or_else(|| ledger.capacity())?;
    ledger.derived(extent)?;
    ledger.visits(count)?;
    ledger.bytes(extent)?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| ledger.capacity())?;
    Ok(values)
}
/// Full eight-field comparison admission, including required null/model tags and native spelling.
fn pin_work(pin: &SourcePinV3, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let SourcePinV3 {
        artifact_key,
        kind: _,
        raw_sha256,
        byte_length: _,
        schema_identity,
        validation_profile,
        native_model: _,
        native_root_uuid,
    } = pin;
    ledger.visits(8)?;
    ledger.bytes(4 * size_of::<u64>())?;
    for value in [
        artifact_key.as_str(),
        raw_sha256.as_str(),
        validation_profile.as_str(),
        schema_identity.as_deref().unwrap_or(""),
        native_root_uuid.as_deref().unwrap_or(""),
    ] {
        ledger.bytes(value.len())?;
    }
    Ok(())
}
/// Preserve every genuine public pin field and option, with no DTO clone shortcut.
fn pins(
    closure: &CurrentAuthoringPlanClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePinV3>, ContractError> {
    let mut values = list(closure.source_pins().len(), ledger)?;
    for pin in closure.source_pins() {
        checkpoint(ledger, control)?;
        ledger.visits(1)?;
        values.push(SourcePinV3 {
            artifact_key: copy(&pin.artifact_key, ledger)?,
            kind: pin.kind,
            raw_sha256: copy(&pin.raw_sha256, ledger)?,
            byte_length: pin.byte_length,
            schema_identity: copy_optional(pin.schema_identity.as_deref(), ledger)?,
            validation_profile: copy(&pin.validation_profile, ledger)?,
            native_model: pin.native_model,
            native_root_uuid: copy_optional(pin.native_root_uuid.as_deref(), ledger)?,
        });
    }
    Ok(values)
}
/// Complete actual stored-plan raw identity, separated from native N and never supplied by a caller.
fn plan_hash<'owner>(
    closure: &'owner CurrentAuthoringPlanClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'owner str, ContractError> {
    let mut result = None;
    for pin in closure.source_pins() {
        checkpoint(ledger, control)?;
        ledger.visits(1)?;
        if pin.kind == SourceKindV3::StoredPlan {
            if result.is_some() {
                return Err(ContractError::Binding);
            }
            result = Some(pin.raw_sha256.as_str());
        }
    }
    result.ok_or(ContractError::Binding)
}
/// Materialize only complete borrowed N descriptors after full storage admission.
fn native_digest(
    closure: &CurrentAuthoringPlanClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    let mut rows = list::<NativeProvenance<'_>>(closure.native_provenance().len(), ledger)?;
    for row in closure.native_provenance() {
        checkpoint(ledger, control)?;
        ledger.visits(1)?;
        rows.push(row);
    }
    hash_v3::native_provenance(&rows, ledger, control)
}
/// Canonical explicit operation headers, with complete parse/format scratch admitted first.
fn headers(
    queue_id: &str,
    created_at: &str,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    for value in [queue_id, created_at] {
        let extent = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
        ledger.bytes(extent)?;
        ledger.derived(128)?;
        ledger.visits(1)?;
    }
    super::validate::uuid(queue_id)?;
    super::validate::time(created_at)
}
/// Fully derive the one complete item; not-yet-filled scalar locals cannot escape this factory.
fn item(
    closure: &CurrentAuthoringPlanClosure,
    request: super::wire_v3::AuthoringInitItem,
    selected: &super::wire_v3::ReviewPolicy,
    pins: &[SourcePinV3],
    queue_id: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ReviewItemV3, ContractError> {
    ledger.derived(size_of::<ReviewItemV3>() + size_of::<ContextSnapshot>())?;
    let mut source_keys = list(pins.len(), ledger)?;
    for pin in pins {
        checkpoint(ledger, control)?;
        source_keys.push(copy(&pin.artifact_key, ledger)?);
    }
    let mut reasons = list(1, ledger)?;
    reasons.push(copy("authoring-plan-current", ledger)?);
    let native = native_digest(closure, ledger, control)?;
    let project = &closure.facts().loaded.project;
    let stored_plan = plan_hash(closure, ledger, control)?;
    let mut item = ReviewItemV3 {
        key: request.key,
        item_id: String::new(),
        domain: DomainV3::AuthoringPlan,
        adapter_version: copy(ADAPTER, ledger)?,
        subject_id: hash_v3::subject_id(&project.project_key, ledger, control)?,
        requested_action: RequestedAction::ReReview,
        source_keys,
        subject_sha256: hash_v3::subject(
            &project.project_key,
            &project.as_of,
            &native,
            stored_plan,
            pins,
            ledger,
            control,
        )?,
        context: ContextSnapshot { reason_codes: reasons, related_subject_ids: list(0, ledger)? },
        context_sha256: String::new(),
        policy_key: request.policy_key,
        policy_sha256: String::new(),
        author_keys: request.author_keys,
        assignments: request.assignments,
        due_at: request.due_at,
        allowed_dispositions: list(5, ledger)?,
    };
    item.allowed_dispositions.extend([
        Disposition::Approve,
        Disposition::Reject,
        Disposition::RequestChanges,
        Disposition::Abstain,
        Disposition::Superseded,
    ]);
    item.context_sha256 = hash_v3::context(pins, &item.context, ledger, control)?;
    item.policy_sha256 = hash_v3::policy(pins, selected, &item, ledger, control)?;
    item.item_id = hash_v3::item_id(queue_id, &item, pins, ledger, control)?;
    Ok(item)
}
/// Prepare genuine current Init output from only the sealed owner and explicit review headers.
pub(crate) fn prepare<'owner>(
    closure: &'owner CurrentAuthoringPlanClosure,
    queue_id: &str,
    created_at: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedAuthoringQueue<'owner>, ContractError> {
    phase(ledger, control, |ledger, control| {
        headers(queue_id, created_at, ledger)?;
        closure.verify_inputs(ledger, control)?;
        let policy_raw = init_original(closure, ledger)?;
        let mut request = super::authoring_init_policy::decode(policy_raw, ledger, control)?;
        let selected_item = request.items.pop().ok_or(ContractError::Invalid)?;
        if let Some(due) = &selected_item.due_at {
            let due_extent =
                due.len().checked_add(created_at.len()).ok_or_else(|| ledger.capacity())?;
            ledger.bytes(due_extent)?;
            ledger.visits(2)?;
            if due.as_str() <= created_at {
                return Err(ContractError::Invalid);
            }
        }
        let pins = pins(closure, ledger, control)?;
        let item =
            item(closure, selected_item, &request.policies[0], &pins, queue_id, ledger, control)?;
        ledger
            .derived(size_of::<QueueDocumentV3>() + size_of::<PreparedAuthoringQueue<'owner>>())?;
        let mut items = list(1, ledger)?;
        items.push(item);
        let document = QueueDocumentV3 {
            schema_version: copy("forge.review-queue/3", ledger)?,
            identity_disclaimer: copy(IDENTITY_DISCLAIMER, ledger)?,
            sensitivity: Sensitivity::IdsAndHashes,
            queue_id: copy(queue_id, ledger)?,
            created_at: copy(created_at, ledger)?,
            source_pins: pins,
            roles: request.roles,
            reviewers: request.reviewers,
            policies: request.policies,
            items,
        };
        super::decode_v3::queue(&document, ledger, control)?;
        closure.verify_inputs(ledger, control)?;
        Ok(PreparedAuthoringQueue { closure, policy_raw, document })
    })
}
