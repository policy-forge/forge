//! A-authored genuine Authoring Queue/3 binding over one complete native owner.
//! Every actual Queue/Response allocation stays borrowed; loaded labels confer no proof.
use super::authoring_capture::CurrentAuthoringPlanClosure;
use super::authoring_response_binding::compared_pins;
use super::capture::authoring::RegisteredAuthoringOriginals;
use super::chain::{compare, reserved};
use super::chain_v3::visit;
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::phase;
use super::decode_v3::{self, DecodedV3};
use super::hash_v3::{self, NativeProvenance};
use super::wire_v3::{QueueDocumentV3, ResponseDocumentV3, SourceKindV3};
use crate::workspace::preparation::WorkControl;

/// Private pending data retaining genuine complete native owner and actual raw inputs.
/// No caller Rc/index/list/hash or Serialize/Clone/currentness flag constructor exists.
pub(crate) struct PendingAuthoringBinding<'native> {
    /// Sole complete N/S/U owner, issued by actual receiver preparation and sealing.
    closure: &'native CurrentAuthoringPlanClosure,
    /// Internally observed whole input Queue and ordered Response occurrence registry.
    originals: RegisteredAuthoringOriginals<'native>,
    /// Inert declarations decoded from the actual held Queue allocation.
    queue: DecodedV3<'native, QueueDocumentV3>,
    /// Every actual Response holder in original order, including duplicate occurrences.
    responses: Vec<DecodedV3<'native, ResponseDocumentV3>>,
}
impl<'native> PendingAuthoringBinding<'native> {
    /// Borrow the genuine native owner without detaching physical lifetime.
    pub(crate) fn closure(&self) -> &'native CurrentAuthoringPlanClosure {
        self.closure
    }
    /// Borrow only the actual complete internally issued operation registry.
    pub(crate) fn originals(&self) -> &RegisteredAuthoringOriginals<'native> {
        &self.originals
    }
    /// Borrow ordinary declarations and the full actual Queue allocation.
    pub(crate) fn queue(&self) -> &DecodedV3<'native, QueueDocumentV3> {
        &self.queue
    }
    /// Borrow every decoded actual Response occurrence; never a selected subset.
    pub(crate) fn responses(&self) -> &[DecodedV3<'native, ResponseDocumentV3>] {
        &self.responses
    }
    /// Repeat exact original registrations and complete physical U with ordinary postfences.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            self.originals.verify_review_originals(ledger, control)?;
            self.closure.verify_inputs(ledger, control)
        })
    }
}
/// Admit complete private native provenance descriptors from the actual owner only.
/// Returned ordinary digest is not a capability or substitute for physical U.
pub(super) fn native_digest(
    closure: &CurrentAuthoringPlanClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        let actual = closure.native_provenance();
        let mut rows: Vec<NativeProvenance<'_>> = reserved(actual.len(), ledger)?;
        for row in actual {
            visit(ledger, control)?;
            ledger.bytes(std::mem::size_of::<NativeProvenance<'_>>())?;
            rows.push(row);
        }
        hash_v3::native_provenance(&rows, ledger, control)
    })
}
/// Close full native identity/pins and all ordinary context/policy/item hashes.
fn native_queue(
    closure: &CurrentAuthoringPlanClosure,
    queue: &DecodedV3<'_, QueueDocumentV3>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let document = queue.document();
        if document.items.len() != 1 || document.policies.len() != 1 {
            return Err(ContractError::Binding);
        }
        let item = &document.items[0];
        let pins = closure.source_pins();
        compared_pins(&document.source_pins, pins, ledger)?;
        if document.source_pins != pins || item.source_keys.len() != pins.len() {
            return Err(ContractError::Binding);
        }
        let mut stored_plan = None;
        for (key, pin) in item.source_keys.iter().zip(pins) {
            visit(ledger, control)?;
            if !compare(key, &pin.artifact_key, ledger)?.is_eq() {
                return Err(ContractError::Binding);
            }
            ledger.bytes(std::mem::size_of::<SourceKindV3>())?;
            if pin.kind == SourceKindV3::StoredPlan {
                if stored_plan.is_some() {
                    return Err(ContractError::Binding);
                }
                stored_plan = Some(pin.raw_sha256.as_str());
            }
        }
        for reason in &item.context.reason_codes {
            visit(ledger, control)?;
            ledger.bytes(reason.len())?;
        }
        for subject in &item.context.related_subject_ids {
            visit(ledger, control)?;
            ledger.bytes(subject.len())?;
        }
        if item.context.reason_codes.len() != 1
            || item.context.reason_codes[0] != "authoring-plan-current"
            || !item.context.related_subject_ids.is_empty()
        {
            return Err(ContractError::Binding);
        }
        let project = &closure.facts().loaded.project;
        let identity = hash_v3::subject_id(&project.project_key, ledger, control)?;
        let provenance = native_digest(closure, ledger, control)?;
        let subject = hash_v3::subject(
            &project.project_key,
            &project.as_of,
            &provenance,
            stored_plan.ok_or(ContractError::Binding)?,
            pins,
            ledger,
            control,
        )?;
        let context = hash_v3::context(pins, &item.context, ledger, control)?;
        let policy = hash_v3::policy(pins, &document.policies[0], item, ledger, control)?;
        let item_id = hash_v3::item_id(&document.queue_id, item, pins, ledger, control)?;
        if !compare(&identity, &item.subject_id, ledger)?.is_eq()
            || !compare(&subject, &item.subject_sha256, ledger)?.is_eq()
            || !compare(&context, &item.context_sha256, ledger)?.is_eq()
            || !compare(&policy, &item.policy_sha256, ledger)?.is_eq()
            || !compare(&item_id, &item.item_id, ledger)?.is_eq()
        {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Decode the actual Queue and every actual Response after genuine complete capture sealing.
/// Foreign/stale/unassigned/future/late ordinary evidence remains available to policy.
pub(crate) fn prepare<'native>(
    closure: &'native CurrentAuthoringPlanClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingAuthoringBinding<'native>, ContractError> {
    phase(ledger, control, |ledger, control| {
        closure.verify_inputs(ledger, control)?;
        let originals = closure.originals()?;
        originals.verify_review_originals(ledger, control)?;
        let raw = originals.queue_raw().ok_or(ContractError::Binding)?;
        let queue = decode_v3::decode_queue(raw, ledger, control)?;
        native_queue(closure, &queue, ledger, control)?;
        let actual = originals.responses();
        let mut responses = reserved(actual.len(), ledger)?;
        for raw in actual {
            visit(ledger, control)?;
            let decoded = decode_v3::decode_response(raw, ledger, control)?;
            ledger.bytes(std::mem::size_of::<DecodedV3<'_, ResponseDocumentV3>>())?;
            responses.push(decoded);
        }
        originals.verify_review_originals(ledger, control)?;
        closure.verify_inputs(ledger, control)?;
        ledger.derived(std::mem::size_of::<PendingAuthoringBinding<'_>>())?;
        Ok(PendingAuthoringBinding { closure, originals, queue, responses })
    })
}
