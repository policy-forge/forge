//! Six explicit-path portable review workflows over genuine held originals.
//! Each invocation accepts one cooperative control before IO and keeps one ledger
//! through capture, native preparation, decoding, projection and its output fence.
//! Asserted keys/times and review quorum never promote native domain approval.
//! Publication/stdout failures are fixed diagnostics; no input prose/path escapes.

use super::adapters::{applicability, mapping};
use super::capture::{HeldReviewInputs, Pool, ReviewCapture, ReviewControl};
use super::chain::{compare, reserved, visit};
use super::decode::{self, ContractError, ContractLedger, Decoded};
use super::finalize::NativeReview;
use super::merge as policy_merge;
use super::wire::{
    Assignment, ContextSnapshot, Disposition, Domain, IDENTITY_DISCLAIMER, QueueDocument,
    RequestedAction, ResponseDocument, ReviewItem, ReviewPolicy, Reviewer, RoleDefinition,
    Sensitivity, SourcePin, SupersessionReference,
};
use super::{
    applicability_capture, encode, finalize, html, mapping_capture, notifications, validate,
};
use crate::evidence_capture::CaptureRole;
use crate::workspace::preparation::WorkControl;
use serde::Deserialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Fixed command failures preserve contract stops without exposing native/parser prose.
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub(crate) enum CommandError {
    /// Exact first structural/capacity/control failure retained under the sole ledger.
    #[error("{0}")]
    Contract(#[from] ContractError),
    /// Native no-replace publication refused or failed; a late durability failure
    /// can leave the complete output, so this variant does not assert no write.
    #[error("review output publication failed")]
    Publication,
    /// Caller-owned stdout/writer failed and can already contain a partial result.
    #[error("review status output failed")]
    Output,
}

/// Explicit first-workflow queue declaration inputs; every path is root confined.
pub(crate) struct InitOptions<'a> {
    /// Actual project root; no workspace discovery or implicit current directory.
    pub(crate) project_root: &'a Path,
    /// Closed domain source locator captured as one Auxiliary original.
    pub(crate) sources: &'a Path,
    /// Closed private forge.review-init/1 declaration, not source or approval proof.
    pub(crate) policy: &'a Path,
    /// Explicit asserted canonical queue revision UUID.
    pub(crate) queue_id: &'a str,
    /// Explicit asserted canonical UTC-second creation time.
    pub(crate) created_at: &'a str,
    /// Confined new destination reserved before any source read; never overwritten.
    pub(crate) output: &'a Path,
}

/// Optional explicit self-supersession pair; omission never guesses a predecessor.
pub(crate) struct Supersedes<'a> {
    /// Asserted previous response UUID, structurally checked by the closed decoder.
    pub(crate) response_id: &'a str,
    /// Full exact previous original hash, not a reconstructed typed-document hash.
    pub(crate) raw_sha256: &'a str,
}

/// Explicit private immutable response inputs; no proposed domain edit route exists.
pub(crate) struct RespondOptions<'a> {
    /// Actual root for the queue, rationale and reserved destination.
    pub(crate) project_root: &'a Path,
    /// Exact actual queue original; neither reformatted nor looked up from an index.
    pub(crate) queue: &'a Path,
    /// Actual UTF-8 private rationale original, at most8 KiB; empty only where policy permits.
    pub(crate) rationale_file: &'a Path,
    /// Explicit queue-local selected item key.
    pub(crate) item_key: &'a str,
    /// Explicit asserted reviewer identity key.
    pub(crate) reviewer_key: &'a str,
    /// Explicit asserted reviewer role; assignment/author separation is checked.
    pub(crate) reviewer_role: &'a str,
    /// Explicit immutable declaration, including abstention or self-supersession.
    pub(crate) disposition: Disposition,
    /// Explicit asserted canonical UTC-second response time.
    pub(crate) responded_at: &'a str,
    /// Explicit canonical response UUID; no random generation or inferred sequence.
    pub(crate) response_id: &'a str,
    /// Explicit abstention code or explicit absence; no default rationale substitution.
    pub(crate) abstention_reason: Option<&'a str>,
    /// Complete explicit prior ID/hash pair or explicit absence.
    pub(crate) supersedes: Option<Supersedes<'a>>,
    /// Confined new response file; actual native no-replace publication only.
    pub(crate) output: &'a Path,
}

/// Complete explicit current review inputs shared by merge and status.
pub(crate) struct CurrentOptions<'a> {
    /// Actual root for every source/queue/response and optional reserved output.
    pub(crate) project_root: &'a Path,
    /// Exact closed native locator; all native sources enter this same capture.
    pub(crate) sources: &'a Path,
    /// Exact complete queue original.
    pub(crate) queue: &'a Path,
    /// Complete explicit response occurrence list; same-path repeats remain registrations.
    pub(crate) responses: &'a [PathBuf],
    /// Explicit asserted policy evaluation time, never inferred from wall clock.
    pub(crate) as_of: &'a str,
}

/// Explicit recorded export, with no native currentness claim from historical bytes.
pub(crate) struct ExportHtmlOptions<'a> {
    /// Actual root for the recorded input and reserved new output.
    pub(crate) project_root: &'a Path,
    /// Complete actual closed dispositions original, up to 32 MiB in `Pool::Recorded`.
    pub(crate) dispositions: &'a Path,
    /// Confined immutable HTML destination; no arbitrary URL or remote assets.
    pub(crate) output: &'a Path,
}

/// Explicit recorded queue export; no response, currentness or delivery evaluation.
pub(crate) struct ExportNotificationsOptions<'a> {
    /// Actual project root containing the recorded queue and reserved new output.
    pub(crate) project_root: &'a Path,
    /// Exact complete closed queue original; native pins remain recorded metadata.
    pub(crate) queue: &'a Path,
    /// Confined new local JSON artifact; an existing destination is refused.
    pub(crate) output: &'a Path,
}

/// Private closed first-profile policy request; all native facts are derived separately.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitRequest {
    /// Exact request revision; no auto-upgrade or domain/source fields.
    schema_version: String,
    /// Complete asserted role roster, sorted and finite under queue validation.
    roles: Vec<RoleDefinition>,
    /// Complete asserted reviewer/role membership roster.
    reviewers: Vec<Reviewer>,
    /// Complete declared quorum policies; neither native approval nor authentication.
    policies: Vec<ReviewPolicy>,
    /// Explicit selected native subjects and declared assignment/deadline policy.
    items: Vec<InitItem>,
}

/// Closed request item contains declaration fields only, never source/hash/proof input.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitItem {
    /// Sorted queue-local key, distinct from actual selected native subject UUID.
    key: String,
    /// Exact native map UUID or applicability control token selected from the complete closure.
    subject_id: String,
    /// Existing declared policy key.
    policy_key: String,
    /// Explicit asserted authors whose reviewer eligibility must be excluded.
    author_keys: Vec<String>,
    /// Explicit complete declared reviewer/role assignments.
    assignments: Vec<Assignment>,
    /// Required nullable explicit UTC-second due time; missing is not null.
    due_at: Option<String>,
}

/// Pending native results can seal only against their genuine common capture owner.
enum PendingNative {
    /// Actual maintained Mapping product and current recorded lifecycle tuple.
    Mapping(mapping_capture::PendingMappingClosure),
    /// Actual full applicability report and current recorded lifecycle tuple.
    Applicability(Box<applicability_capture::PendingApplicabilityClosure>),
}

/// Closed owned domain capture variants; no detached input or success constructor exists.
enum NativeClosure {
    /// Genuine once-captured Mapping source closure.
    Mapping(mapping_capture::ApprovedMappingClosure),
    /// Genuine once-captured full applicability report-source closure.
    Applicability(Box<applicability_capture::ApprovedApplicabilityClosure>),
}

impl PendingNative {
    /// Transfer the exact pending original leases into one complete held proof owner.
    fn seal(
        self,
        held: Rc<HeldReviewInputs>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<NativeClosure, ContractError> {
        match self {
            Self::Mapping(value) => value.seal(held, ledger, control).map(NativeClosure::Mapping),
            Self::Applicability(value) => {
                let closure = (*value).seal(held, ledger, control)?;
                boxed(closure, ledger, control).map(NativeClosure::Applicability)
            }
        }
    }
}

/// Selected domain facts borrow the actual owned sealed source closure.
enum PreparedNative<'a> {
    /// Selected native map assertions from the same complete actual source set.
    Mapping(mapping::PreparedMappingReview<'a>),
    /// Selected explicit control decisions; omitted decisions remain denominators.
    Applicability(applicability::PreparedApplicabilityReview<'a>),
}

impl NativeClosure {
    /// Read the internally selected domain from the genuine successful variant.
    fn domain(&self) -> Domain {
        match self {
            Self::Mapping(_) => Domain::MappingAssertion,
            Self::Applicability(_) => Domain::ApplicabilityDecision,
        }
    }

    /// Prepare selected facts through the matching genuine captured domain adapter.
    fn prepare<'a>(
        &'a self,
        selected: &[&str],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<PreparedNative<'a>, ContractError> {
        match self {
            Self::Mapping(value) => {
                mapping::prepare(value, selected, ledger, control).map(PreparedNative::Mapping)
            }
            Self::Applicability(value) => applicability::prepare(value, selected, ledger, control)
                .map(PreparedNative::Applicability),
        }
    }
}

impl PreparedNative<'_> {
    /// Borrow the actual sealed adapter for the common owner/cohort finalizer.
    fn view(&self) -> NativeReview<'_, '_> {
        match self {
            Self::Mapping(value) => NativeReview::Mapping(value),
            Self::Applicability(value) => NativeReview::Applicability(value),
        }
    }
}

/// Read only the exact closed marker from the actual admitted private locator original.
fn source_domain(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Domain, ContractError> {
    let value = mapping_capture::strict_value(raw, 1_048_576, ledger, control)?;
    match value.get("schema_version").and_then(Value::as_str) {
        Some("forge.review-source-locator/1") => Ok(Domain::MappingAssertion),
        Some("forge.review-applicability-locator/1") => Ok(Domain::ApplicabilityDecision),
        _ => Err(ContractError::Invalid),
    }
}

/// Consume one exact marker and the complete selected factory, without fallback domains.
fn prepare_native(
    capture: &mut ReviewCapture,
    locator: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingNative, ContractError> {
    match source_domain(capture.bytes(locator)?, ledger, control)? {
        Domain::MappingAssertion => {
            mapping_capture::prepare(capture, locator, ledger, control).map(PendingNative::Mapping)
        }
        Domain::ApplicabilityDecision => {
            let pending = applicability_capture::prepare(capture, locator, ledger, control)?;
            boxed(pending, ledger, control).map(PendingNative::Applicability)
        }
    }
}

/// Admit the complete fixed wrapper payload before applicability enum indirection grows.
/// Original leases/owners move intact; logical credit is monotonic and never released.
fn boxed<T>(
    value: T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Box<T>, ContractError> {
    ledger.checkpoint(control)?;
    let payload = std::mem::size_of::<T>();
    ledger.bytes(payload)?;
    ledger.derived(payload.checked_add(32).ok_or(ContractError::Capacity)?)?;
    Ok(Box::new(value))
}

/// Current result sink; the caller supplies only a local new file or an actual writer.
enum CurrentDestination<'a> {
    /// Confined reserved no-replace output.
    File(&'a Path),
    /// Caller-owned readonly stdout stream, with partial-IO failure explicitly possible.
    Writer(&'a mut dyn Write),
}

/// Initialize a complete queue from actual recorded Approved/current domain source facts.
/// No caller policy request can provide a source pin, fingerprint or success proof.
pub(crate) fn init(
    options: &InitOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    init_inner(options, &mut ledger, &mut control)
}

/// Retain all actual queue source/declaration owners through the final guarded publisher.
fn init_inner(
    options: &InitOptions<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    ledger.checkpoint(control)?;
    ledger.bytes(
        options
            .queue_id
            .len()
            .checked_add(options.created_at.len())
            .ok_or(ContractError::Capacity)?,
    )?;
    if options.queue_id.len() != 36 || options.created_at.len() != 20 {
        return Err(ContractError::Invalid.into());
    }
    validate::uuid(options.queue_id)?;
    validate::time(options.created_at)?;
    let mut capture = ReviewCapture::new(options.project_root, &[options.output], ledger, control)?;
    let sources = auxiliary(&mut capture, options.sources, 1_048_576, ledger, control)?;
    let policy = auxiliary(&mut capture, options.policy, 1_048_576, ledger, control)?;
    let pending = prepare_native(&mut capture, sources, ledger, control)?;
    let held = owned(capture, ledger, control)?;
    let closure = pending.seal(Rc::clone(&held), ledger, control)?;
    let domain = closure.domain();
    let request = init_request_domain(held.bytes(policy)?, domain, ledger, control)?;
    let mut selected = reserved(request.items.len(), ledger)?;
    for item in &request.items {
        visit(ledger, control)?;
        selected.push(item.subject_id.as_str());
    }
    sort_subjects_domain(&mut selected, domain, ledger, control)?;
    let prepared = closure.prepare(&selected, ledger, control)?;
    let native = prepared.view();
    ledger.visits(1)?;
    if native.selected_count() != selected.len() {
        return Err(ContractError::Binding.into());
    }
    drop(selected);
    let queue = build_queue(request, options, native, ledger, control)?;
    let output = encode::queue(&queue, ledger, control)?;
    let closed = decode::decode_queue(&output, ledger, control)?;
    native.bind_queue(&closed, ledger, control)?;
    native.verify_inputs(ledger, control)?;
    publish(options.project_root, options.output, &output, ledger, control, |ledger, control| {
        native.verify_inputs(ledger, control)
    })
}

/// Create one immutable private assertion from actual queue/rationale originals.
/// Only the exact declared policy binding is checked; no domain currentness is asserted.
pub(crate) fn respond(
    options: &RespondOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    respond_inner(options, &mut ledger, &mut control)
}

/// Preserve publication's fixed command diagnostic separately from contract errors.
fn respond_inner(
    options: &RespondOptions<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut capture = ReviewCapture::new(options.project_root, &[options.output], ledger, control)?;
    let queue_slot = capture.required(
        options.queue,
        CaptureRole::ReviewQueue,
        Pool::Queue,
        10_485_760,
        ledger,
        control,
    )?;
    let rationale_slot = auxiliary(&mut capture, options.rationale_file, 8192, ledger, control)?;
    let held = owned(capture, ledger, control)?;
    let queue = decode::decode_queue(held.bytes(queue_slot)?, ledger, control)?;
    let response = build_response(options, &queue, held.bytes(rationale_slot)?, ledger, control)?;
    let output = encode::response(&response, ledger, control)?;
    let closed = decode::decode_response(&output, ledger, control)?;
    validate::bind_response(&queue, &closed, ledger, control)?;
    held.verify_inputs(ledger, control)?;
    publish(options.project_root, options.output, &output, ledger, control, |ledger, control| {
        held.verify_inputs(ledger, control)
    })
}

/// Merge complete actual inputs through native gating, then publish a new current record.
pub(crate) fn merge(
    options: &CurrentOptions<'_>,
    output: &Path,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    current(options, CurrentDestination::File(output), caller)
}

/// Emit complete genuine-current JSON to an actual caller writer without project mutation.
/// Writer/syscall failures can leave partial stdout and are never retried or called no-write.
pub(crate) fn status(
    options: &CurrentOptions<'_>,
    writer: &mut dyn Write,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    current(options, CurrentDestination::Writer(writer), caller)
}

/// Actual sealed current input cohort; constructible here only by real capture/factory ports.
struct CurrentCapture {
    /// Same actual shared owner passed to native sealing and eventual finalization.
    held: Rc<HeldReviewInputs>,
    /// Genuine opaque native closure, never a caller-provided hash or Boolean.
    closure: NativeClosure,
    /// Actual successful Queue-pool registration.
    queue_index: usize,
    /// Complete successful response occurrence cohort in caller order, including repeats.
    response_indices: Vec<usize>,
}

/// Keep every local owner alive through one final native/full-original output fence.
fn current(
    options: &CurrentOptions<'_>,
    destination: CurrentDestination<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    ledger.checkpoint(&mut control)?;
    ledger.bytes(options.as_of.len())?;
    validate::time(options.as_of)?;
    if options.responses.len() > 10_000 {
        return Err(ledger.capacity().into());
    }
    let output = match &destination {
        CurrentDestination::File(path) => Some(*path),
        CurrentDestination::Writer(_) => None,
    };
    let captured = capture_current(options, output, &mut ledger, &mut control)?;
    emit_current(options, &captured, destination, &mut ledger, &mut control)
}

/// Admit all explicit queue/response/source/output originals within one genuine session.
fn capture_current(
    options: &CurrentOptions<'_>,
    output: Option<&Path>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentCapture, ContractError> {
    let mut capture = ReviewCapture::new(options.project_root, output.as_slice(), ledger, control)?;
    let sources = auxiliary(&mut capture, options.sources, 1_048_576, ledger, control)?;
    let queue_index = capture.required(
        options.queue,
        CaptureRole::ReviewQueue,
        Pool::Queue,
        10_485_760,
        ledger,
        control,
    )?;
    let response_indices = capture_responses(&mut capture, options.responses, ledger, control)?;
    let pending = prepare_native(&mut capture, sources, ledger, control)?;
    let held = owned(capture, ledger, control)?;
    let closure = pending.seal(Rc::clone(&held), ledger, control)?;
    ledger.derived(std::mem::size_of::<CurrentCapture>())?;
    ledger.checkpoint(control)?;
    Ok(CurrentCapture { held, closure, queue_index, response_indices })
}

/// Decode actual held allocations, gate native policy, then encode and fence the full result.
fn emit_current(
    options: &CurrentOptions<'_>,
    captured: &CurrentCapture,
    mut destination: CurrentDestination<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let queue = decode::decode_queue(captured.held.bytes(captured.queue_index)?, ledger, control)?;
    let selected = queue_subjects(queue.document(), captured.closure.domain(), ledger, control)?;
    let prepared = captured.closure.prepare(&selected, ledger, control)?;
    let native = prepared.view();
    let responses = decode_responses(&captured.held, &captured.response_indices, ledger, control)?;
    let pending = policy_merge::prepare_policy(&queue, &responses, options.as_of, ledger, control)?;
    let result = match native {
        NativeReview::Mapping(mapping) => finalize::finalize(
            &captured.held,
            captured.queue_index,
            &captured.response_indices,
            &pending,
            mapping,
            ledger,
            control,
        ),
        NativeReview::Applicability(_) => finalize::finalize_native(
            &captured.held,
            captured.queue_index,
            &captured.response_indices,
            &pending,
            native,
            ledger,
            control,
        ),
    }?;
    let bytes = encode::dispositions(result.document(), ledger, control)?;
    // Validate the actual bounded encoded publication form against its original
    // queue declarations under the unchanged ledger; this grants no native proof.
    let encoded = decode::decode_dispositions(&bytes, ledger, control)?;
    validate::bind_dispositions(&queue, &encoded, ledger, control)?;
    result.verify_inputs(ledger, control)?;
    match &mut destination {
        CurrentDestination::File(path) => {
            publish(options.project_root, path, &bytes, ledger, control, |ledger, control| {
                result.verify_inputs(ledger, control)
            })
        }
        CurrentDestination::Writer(writer) => {
            ledger.bytes(bytes.len())?;
            result.verify_inputs(ledger, control)?;
            writer.write_all(&bytes).map_err(|_| CommandError::Output)
        }
    }
}

/// Render only actual captured recorded dispositions; its labels grant no fresh authority.
/// `Pool::Recorded/ReviewDispositions` is a required genuine Root bridge, not a local stub.
pub(crate) fn export_html(
    options: &ExportHtmlOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    let mut capture =
        ReviewCapture::new(options.project_root, &[options.output], &mut ledger, &mut control)?;
    let slot = capture.required(
        options.dispositions,
        CaptureRole::ReviewDispositions,
        Pool::Recorded,
        33_554_432,
        &mut ledger,
        &mut control,
    )?;
    let held = owned(capture, &mut ledger, &mut control)?;
    let recorded = decode::decode_dispositions(held.bytes(slot)?, &mut ledger, &mut control)?;
    let bytes = html::render(&recorded, &mut ledger, &mut control)?;
    held.verify_inputs(&mut ledger, &mut control)?;
    publish(
        options.project_root,
        options.output,
        &bytes,
        &mut ledger,
        &mut control,
        |ledger, control| held.verify_inputs(ledger, control),
    )
}

/// Export every explicit assignment or unassigned row from a captured recorded queue.
/// One caller/ledger/held original remains alive through guarded no-replace publication.
/// No native currentness, evaluated quorum, recipient identity or delivery is inferred.
pub(crate) fn export_notifications(
    options: &ExportNotificationsOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    let mut capture =
        ReviewCapture::new(options.project_root, &[options.output], &mut ledger, &mut control)?;
    let slot = capture.required(
        options.queue,
        CaptureRole::ReviewQueue,
        Pool::Queue,
        10_485_760,
        &mut ledger,
        &mut control,
    )?;
    let held = owned(capture, &mut ledger, &mut control)?;
    let prepared = notifications::prepare(&held, slot, &mut ledger, &mut control)?;
    prepared.verify_inputs(&mut ledger, &mut control)?;
    publish(
        options.project_root,
        options.output,
        prepared.bytes(),
        &mut ledger,
        &mut control,
        |ledger, control| prepared.verify_inputs(ledger, control),
    )
}

/// Capture one private actual original with a smaller caller ceiling where required.
fn auxiliary(
    capture: &mut ReviewCapture,
    path: &Path,
    maximum: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    capture.required(
        path,
        CaptureRole::ReviewPrivateConfig,
        Pool::Auxiliary,
        maximum,
        ledger,
        control,
    )
}

/// Finish once, reserving the actual shared owner wrapper before allocation.
fn owned(
    capture: ReviewCapture,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Rc<HeldReviewInputs>, ContractError> {
    ledger.checkpoint(control)?;
    ledger.derived(
        std::mem::size_of::<HeldReviewInputs>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .ok_or(ContractError::Capacity)?,
    )?;
    Ok(Rc::new(capture.finish()))
}

/// Capture complete caller-listed occurrences before native closure sealing.
fn capture_responses(
    capture: &mut ReviewCapture,
    paths: &[PathBuf],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<usize>, ContractError> {
    let mut indices = reserved(paths.len(), ledger)?;
    for path in paths {
        visit(ledger, control)?;
        indices.push(capture.required(
            path.as_path(),
            CaptureRole::ReviewResponse,
            Pool::Response,
            1_048_576,
            ledger,
            control,
        )?);
    }
    Ok(indices)
}

/// Borrow each exact actual original in registration order, including same-path duplicates.
fn decode_responses<'a>(
    held: &'a HeldReviewInputs,
    indices: &[usize],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<Decoded<'a, ResponseDocument>>, ContractError> {
    let mut responses = reserved(indices.len(), ledger)?;
    for index in indices {
        visit(ledger, control)?;
        responses.push(decode::decode_response(held.bytes(*index)?, ledger, control)?);
    }
    Ok(responses)
}

/// Repeat the genuine fence immediately before native no-replace rename, preserving its cause.
/// Late native durability errors may leave a complete file; no rollback/no-write claim occurs.
fn publish(
    root: &Path,
    output: &Path,
    bytes: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    mut fence: impl FnMut(&mut ContractLedger, &mut dyn WorkControl) -> Result<(), ContractError>,
) -> Result<(), CommandError> {
    ledger.checkpoint(control)?;
    ledger.bytes(bytes.len())?;
    let mut cause = None;
    let result = crate::authoring::output::publish_new_file_guarded(root, output, bytes, || {
        match fence(ledger, control).and_then(|()| ledger.checkpoint(control)) {
            Ok(()) => Ok(()),
            Err(error) => {
                cause = Some(error);
                Err(crate::error::ForgeError::Authoring("review output fence refused".into()))
            }
        }
    });
    if let Some(error) = cause {
        return Err(error.into());
    }
    result.map_err(|_| CommandError::Publication)
}

/// Admit private raw declarations before moving any tree into typed containers.
#[cfg(test)]
fn init_request(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<InitRequest, ContractError> {
    init_request_domain(raw, Domain::MappingAssertion, ledger, control)
}

/// Decode declarations for the exact domain selected by the genuine source locator.
fn init_request_domain(
    raw: &[u8],
    domain: Domain,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<InitRequest, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        if raw.len() > 1_048_576 {
            return Err(ledger.capacity());
        }
        if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
            return Err(ContractError::Invalid);
        }
        ledger.bytes(raw.len())?;
        std::str::from_utf8(raw).map_err(|_| ContractError::Invalid)?;
        ledger.bytes(raw.len())?;
        let value = crate::json_strict::parse_value(
            raw,
            "review-init",
            crate::json_strict::Limits { max_depth: 64, max_string_bytes: 65_536 },
        )
        .map_err(|_| ContractError::Invalid)?;
        reserve_request(&value, ledger, control)?;
        request_shape(&value, domain, ledger, control)?;
        ledger.bytes(raw.len())?;
        let request: InitRequest =
            serde_json::from_value(value).map_err(|_| ContractError::Invalid)?;
        if request.schema_version != "forge.review-init/1" {
            return Err(ContractError::Invalid);
        }
        ledger.checkpoint(control)?;
        Ok(request)
    })
}

/// Require exact fields/cardinalities before typed vectors can grow; null `due_at` is explicit.
fn request_shape(
    value: &Value,
    domain: Domain,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    fields(value, &["schema_version", "roles", "reviewers", "policies", "items"], ledger, control)?;
    if value.get("schema_version").and_then(Value::as_str) != Some("forge.review-init/1") {
        return Err(ContractError::Invalid);
    }
    for role in array(value, "roles", 1, 32)? {
        fields(role, &["key"], ledger, control)?;
        text_field(role, "key", 128)?;
    }
    for reviewer in array(value, "reviewers", 1, 100)? {
        fields(reviewer, &["key", "role_keys"], ledger, control)?;
        text_field(reviewer, "key", 128)?;
        token_array(reviewer, "role_keys", 1, 32, 128)?;
    }
    for policy in array(value, "policies", 1, 10_000)? {
        policy_shape(policy, ledger, control)?;
    }
    for item in array(value, "items", 1, 10_000)? {
        item_request_shape(item, domain, ledger, control)?;
    }
    ledger.checkpoint(control)
}

/// Bound one complete selected declaration before typed item/assignment growth.
fn item_request_shape(
    item: &Value,
    domain: Domain,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    fields(
        item,
        &["key", "subject_id", "policy_key", "author_keys", "assignments", "due_at"],
        ledger,
        control,
    )?;
    text_field(item, "key", 128)?;
    text_field(
        item,
        "subject_id",
        match domain {
            Domain::MappingAssertion => 36,
            Domain::ApplicabilityDecision => 256,
        },
    )?;
    text_field(item, "policy_key", 128)?;
    token_array(item, "author_keys", 1, 100, 128)?;
    for assignment in array(item, "assignments", 0, 3200)? {
        fields(assignment, &["reviewer_key", "role_key"], ledger, control)?;
        text_field(assignment, "reviewer_key", 128)?;
        text_field(assignment, "role_key", 128)?;
    }
    let due = item.get("due_at").ok_or(ContractError::Invalid)?;
    if !due.is_null() && due.as_str().is_none_or(|text| text.len() != 20) {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Bound all nested policy relations before their actual closed wire structs are created.
fn policy_shape(
    policy: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    fields(
        policy,
        &[
            "key",
            "seats",
            "substitutions",
            "abstention_rule",
            "empty_abstention_reasons",
            "author_separation",
        ],
        ledger,
        control,
    )?;
    text_field(policy, "key", 128)?;
    for seat in array(policy, "seats", 1, 32)? {
        fields(seat, &["role_key", "count"], ledger, control)?;
        text_field(seat, "role_key", 128)?;
        if !seat
            .get("count")
            .and_then(Value::as_u64)
            .is_some_and(|count| (1..=100).contains(&count))
        {
            return Err(ContractError::Invalid);
        }
    }
    for substitution in array(policy, "substitutions", 0, 3200)? {
        fields(
            substitution,
            &["seat_role", "reviewer_key", "asserted_role", "reason_code"],
            ledger,
            control,
        )?;
        for key in ["seat_role", "reviewer_key", "asserted_role", "reason_code"] {
            text_field(substitution, key, 128)?;
        }
    }
    token_array(policy, "empty_abstention_reasons", 0, 32, 128)?;
    if policy.get("abstention_rule").and_then(Value::as_str) != Some("nonapproving")
        || policy.get("author_separation").and_then(Value::as_str) != Some("declared-keys")
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Check a closed object against required constant fields, admitting all inspected key work.
fn fields(
    value: &Value,
    expected: &[&str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    visit(ledger, control)?;
    let object = value.as_object().ok_or(ContractError::Invalid)?;
    if object.len() != expected.len() {
        return Err(ContractError::Invalid);
    }
    for (key, _) in object {
        let mut found = false;
        for field in expected {
            ledger.bytes(key.len().checked_add(field.len()).ok_or(ContractError::Capacity)?)?;
            ledger.visits(1)?;
            if key == field {
                found = true;
                break;
            }
        }
        if !found {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}

/// Borrow a required complete array after its exact before-growth cardinality admission.
fn array<'a>(
    value: &'a Value,
    field: &str,
    minimum: usize,
    maximum: usize,
) -> Result<&'a [Value], ContractError> {
    let rows = value.get(field).and_then(Value::as_array).ok_or(ContractError::Invalid)?;
    if !(minimum..=maximum).contains(&rows.len()) {
        return Err(ContractError::Invalid);
    }
    Ok(rows)
}

/// Check a required bounded string without allocating or normalizing its original spelling.
fn text_field(value: &Value, field: &str, maximum: usize) -> Result<(), ContractError> {
    let text = value.get(field).and_then(Value::as_str).ok_or(ContractError::Invalid)?;
    if text.is_empty() || text.len() > maximum {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Bound complete declared token lists before typed Vec/String growth.
fn token_array(
    value: &Value,
    field: &str,
    minimum: usize,
    maximum: usize,
    width: usize,
) -> Result<(), ContractError> {
    for row in array(value, field, minimum, maximum)? {
        if row.as_str().is_none_or(|text| text.is_empty() || text.len() > width) {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}

/// Charge every complete bounded tree node/key/text before typed deserialization.
/// Parser/schema internal temporary storage remains qualified, not heap-confined.
fn reserve_request(
    value: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    visit(ledger, control)?;
    // Closed shape inspection performs at most32 fixed-field probes per node.
    // Admit that complete conservative work before the following shape pass.
    ledger.visits(32)?;
    ledger.derived(64)?;
    match value {
        Value::String(text) => {
            ledger.bytes(text.len().checked_mul(32).ok_or(ContractError::Capacity)?)?;
            ledger.derived(text.len())?;
        }
        Value::Array(rows) => {
            for row in rows {
                reserve_request(row, ledger, control)?;
            }
        }
        Value::Object(rows) => {
            for (key, row) in rows {
                ledger.bytes(key.len().checked_mul(32).ok_or(ContractError::Capacity)?)?;
                ledger.derived(key.len())?;
                reserve_request(row, ledger, control)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

/// Precharge a new owned bounded string before allocating/copying its actual complete bytes.
fn copied(
    value: &str,
    maximum: usize,
    ledger: &mut ContractLedger,
) -> Result<String, ContractError> {
    ledger.bytes(value.len())?;
    if value.len() > maximum {
        return Err(ContractError::Invalid);
    }
    ledger.derived(
        value.len().checked_add(std::mem::size_of::<String>()).ok_or(ContractError::Capacity)?,
    )?;
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| ContractError::Capacity)?;
    result.push_str(value);
    Ok(result)
}

/// Copy only complete bounded declared token payloads; no private path/prose projection.
fn strings(
    values: &[String],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<String>, ContractError> {
    let mut output = reserved(values.len(), ledger)?;
    for value in values {
        visit(ledger, control)?;
        output.push(copied(value, 256, ledger)?);
    }
    Ok(output)
}

/// Copy exact whole sealed source pins with every payload admitted before growth.
fn pins(
    values: &[SourcePin],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePin>, ContractError> {
    let mut output = reserved(values.len(), ledger)?;
    for pin in values {
        visit(ledger, control)?;
        output.push(SourcePin {
            artifact_key: copied(&pin.artifact_key, 128, ledger)?,
            model: pin.model,
            native_root_uuid: pin
                .native_root_uuid
                .as_deref()
                .map(|uuid| copied(uuid, 36, ledger))
                .transpose()?,
            raw_sha256: copied(&pin.raw_sha256, 64, ledger)?,
            byte_length: pin.byte_length,
            schema_identity: copied(&pin.schema_identity, 128, ledger)?,
        });
    }
    Ok(output)
}

/// Build a queue using the actual adapter facts, moving declaration storage without clones.
fn build_queue(
    request: InitRequest,
    options: &InitOptions<'_>,
    native: NativeReview<'_, '_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<QueueDocument, ContractError> {
    ledger.checkpoint(control)?;
    let InitRequest { schema_version: _, roles, reviewers, policies, items } = request;
    ledger.derived(std::mem::size_of::<QueueDocument>())?;
    let mut queue = QueueDocument {
        schema_version: copied("forge.review-queue/1", 128, ledger)?,
        identity_disclaimer: copied(IDENTITY_DISCLAIMER, 128, ledger)?,
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: copied(options.queue_id, 36, ledger)?,
        created_at: copied(options.created_at, 20, ledger)?,
        source_pins: pins(native.source_pins(), ledger, control)?,
        roles,
        reviewers,
        policies,
        items: reserved(items.len(), ledger)?,
    };
    strict_policy_keys(&queue.policies, ledger, control)?;
    let facts = fact_views(native, items.len(), ledger, control)?;
    for declaration in items {
        visit(ledger, control)?;
        let fact = fact_for(&facts, &declaration.subject_id, ledger, control)?;
        let mut item =
            native_item(declaration, fact, native.domain(), native.source_pins(), ledger, control)?;
        ledger.checkpoint(control)?;
        item.context_sha256 = validate::context_hash(&queue, &item, ledger)?;
        let policy = policy_for(&queue.policies, &item.policy_key, ledger, control)?;
        ledger.checkpoint(control)?;
        item.policy_sha256 = validate::policy_hash(&queue, policy, &item, ledger)?;
        ledger.checkpoint(control)?;
        item.item_id = validate::item_id(&queue, &item, ledger)?;
        queue.items.push(item);
    }
    validate::queue(&queue, ledger, control)?;
    Ok(queue)
}

/// Bind one declared item to exact native subject/context and complete source keys.
fn native_item(
    declaration: InitItem,
    fact: &FactView<'_>,
    domain: Domain,
    source_pins: &[SourcePin],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ReviewItem, ContractError> {
    let mut source_keys = reserved(source_pins.len(), ledger)?;
    for pin in source_pins {
        visit(ledger, control)?;
        source_keys.push(copied(&pin.artifact_key, 128, ledger)?);
    }
    let context = ContextSnapshot {
        reason_codes: strings(&fact.context.reason_codes, ledger, control)?,
        related_subject_ids: strings(&fact.context.related_subject_ids, ledger, control)?,
    };
    let mut allowed_dispositions = reserved(5, ledger)?;
    allowed_dispositions.extend([
        Disposition::Approve,
        Disposition::Reject,
        Disposition::RequestChanges,
        Disposition::Abstain,
        Disposition::Superseded,
    ]);
    Ok(ReviewItem {
        key: declaration.key,
        item_id: String::new(),
        domain,
        adapter_version: copied(
            match domain {
                Domain::MappingAssertion => "forge.mapping-review/1",
                Domain::ApplicabilityDecision => "forge.applicability-review/1",
            },
            128,
            ledger,
        )?,
        subject_id: declaration.subject_id,
        requested_action: RequestedAction::ReReview,
        source_keys,
        subject_sha256: copied(fact.subject_sha256, 64, ledger)?,
        context,
        context_sha256: String::new(),
        policy_key: declaration.policy_key,
        policy_sha256: String::new(),
        author_keys: declaration.author_keys,
        assignments: declaration.assignments,
        due_at: declaration.due_at,
        allowed_dispositions,
    })
}

/// Require sorted unique policy keys before any binary search; full semantics follow later.
fn strict_policy_keys(
    policies: &[ReviewPolicy],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for pair in policies.windows(2) {
        visit(ledger, control)?;
        if compare(&pair[0].key, &pair[1].key, ledger)? != Ordering::Less {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}

/// Find a declared policy through charged complete comparisons of its sorted key registry.
fn policy_for<'a>(
    policies: &'a [ReviewPolicy],
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a ReviewPolicy, ContractError> {
    let mut low = 0;
    let mut high = policies.len();
    while low < high {
        visit(ledger, control)?;
        let middle = low + (high - low) / 2;
        match compare(&policies[middle].key, key, ledger)? {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Ok(&policies[middle]),
        }
    }
    Err(ContractError::Binding)
}

/// Borrowed minimized fields from actual selected adapter facts; no source or proof is copied.
struct FactView<'a> {
    /// Exact actual native subject identity under its selected domain.
    subject_id: &'a str,
    /// Complete private subject digest, already derived by the actual adapter.
    subject_sha256: &'a str,
    /// Actual minimized context, borrowed until admitted queue copying.
    context: &'a ContextSnapshot,
}

/// Admit the complete selected fact vector before borrowing either adapter's rows.
fn fact_views<'a>(
    native: NativeReview<'a, '_>,
    expected: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<FactView<'a>>, ContractError> {
    visit(ledger, control)?;
    if native.selected_count() != expected {
        return Err(ContractError::Binding);
    }
    let mut facts = reserved(expected, ledger)?;
    macro_rules! borrow_facts {
        ($value:expr) => {
            for fact in $value.facts() {
                visit(ledger, control)?;
                if facts.len() == expected {
                    return Err(ContractError::Binding);
                }
                facts.push(FactView {
                    subject_id: fact.subject_id(),
                    subject_sha256: fact.subject_sha256(),
                    context: fact.context(),
                });
            }
        };
    }
    match native {
        NativeReview::Mapping(value) => {
            borrow_facts!(value);
        }
        NativeReview::Applicability(value) => {
            borrow_facts!(value);
        }
    }
    if facts.len() != expected {
        return Err(ContractError::Binding);
    }
    Ok(facts)
}

/// Find an actual sorted native fact; no fallback to a caller-provided hash/context exists.
fn fact_for<'a>(
    facts: &'a [FactView<'a>],
    id: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a FactView<'a>, ContractError> {
    let mut low = 0;
    let mut high = facts.len();
    while low < high {
        visit(ledger, control)?;
        let middle = low + (high - low) / 2;
        match compare(facts[middle].subject_id, id, ledger)? {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Ok(&facts[middle]),
        }
    }
    Err(ContractError::Binding)
}

/// Preserve queue item order while selecting one sorted unique native-ID cohort.
fn queue_subjects<'a>(
    queue: &'a QueueDocument,
    domain: Domain,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<&'a str>, ContractError> {
    let mut selected = reserved(queue.items.len(), ledger)?;
    for item in &queue.items {
        visit(ledger, control)?;
        if item.domain != domain {
            return Err(ContractError::Binding);
        }
        selected.push(item.subject_id.as_str());
    }
    sort_subjects_domain(&mut selected, domain, ledger, control)?;
    Ok(selected)
}

/// In-place finite heapsort of borrowed IDs with admitted comparisons, then exact uniqueness.
#[cfg(test)]
fn sort_subjects(
    values: &mut [&str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    sort_subjects_domain(values, Domain::MappingAssertion, ledger, control)
}

/// Sort the complete selected cohort using the unchanged domain-specific identity grammar.
fn sort_subjects_domain(
    values: &mut [&str],
    domain: Domain,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    if values.is_empty() || values.len() > 10_000 {
        return Err(ContractError::Invalid);
    }
    for value in values.iter() {
        visit(ledger, control)?;
        ledger.bytes(value.len())?;
        match domain {
            Domain::MappingAssertion => {
                if value.len() != 36 {
                    return Err(ContractError::Invalid);
                }
                validate::uuid(value)?;
            }
            Domain::ApplicabilityDecision => validate::token(value, 256)?,
        }
    }
    for root in (0..values.len() / 2).rev() {
        sift_subjects(values, root, values.len(), ledger, control)?;
    }
    for end in (1..values.len()).rev() {
        visit(ledger, control)?;
        values.swap(0, end);
        sift_subjects(values, 0, end, ledger, control)?;
    }
    for pair in values.windows(2) {
        visit(ledger, control)?;
        if compare(pair[0], pair[1], ledger)? != Ordering::Less {
            return Err(ContractError::Binding);
        }
    }
    Ok(())
}

/// Repair a bounded borrowed-ID heap while charging each complete string comparison.
fn sift_subjects(
    values: &mut [&str],
    mut root: usize,
    end: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    loop {
        visit(ledger, control)?;
        let child =
            root.checked_mul(2).and_then(|n| n.checked_add(1)).ok_or(ContractError::Capacity)?;
        if child >= end {
            return Ok(());
        }
        let right = child + 1;
        let larger =
            if right < end && compare(values[child], values[right], ledger)? == Ordering::Less {
                right
            } else {
                child
            };
        if compare(values[root], values[larger], ledger)? != Ordering::Less {
            return Ok(());
        }
        values.swap(root, larger);
        root = larger;
    }
}

/// Build all private response fields from exact queue binding and actual UTF-8 rationale bytes.
fn build_response(
    options: &RespondOptions<'_>,
    queue: &Decoded<'_, QueueDocument>,
    raw_rationale: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ResponseDocument, ContractError> {
    ledger.checkpoint(control)?;
    if raw_rationale.len() > 8192 {
        return Err(ledger.capacity());
    }
    ledger.bytes(raw_rationale.len())?;
    let rationale = std::str::from_utf8(raw_rationale).map_err(|_| ContractError::Invalid)?;
    let document = queue.document();
    let mut selected = None;
    for item in &document.items {
        visit(ledger, control)?;
        if compare(&item.key, options.item_key, ledger)? == Ordering::Equal {
            selected = Some(item);
        }
    }
    let item = selected.ok_or(ContractError::Binding)?;
    ledger.derived(std::mem::size_of::<ResponseDocument>())?;
    let supersedes = options
        .supersedes
        .as_ref()
        .map(|prior| -> Result<SupersessionReference, ContractError> {
            Ok(SupersessionReference {
                response_id: copied(prior.response_id, 36, ledger)?,
                raw_sha256: copied(prior.raw_sha256, 64, ledger)?,
            })
        })
        .transpose()?;
    let response = ResponseDocument {
        schema_version: copied("forge.review-response/1", 128, ledger)?,
        identity_disclaimer: copied(IDENTITY_DISCLAIMER, 128, ledger)?,
        response_id: copied(options.response_id, 36, ledger)?,
        queue_id: copied(&document.queue_id, 36, ledger)?,
        queue_raw_sha256: copied(queue.raw_sha256(), 64, ledger)?,
        item_key: copied(&item.key, 128, ledger)?,
        item_id: copied(&item.item_id, 36, ledger)?,
        domain: item.domain,
        adapter_version: copied(&item.adapter_version, 128, ledger)?,
        requested_action: item.requested_action,
        source_pins: pins(&document.source_pins, ledger, control)?,
        subject_sha256: copied(&item.subject_sha256, 64, ledger)?,
        context_sha256: copied(&item.context_sha256, 64, ledger)?,
        policy_sha256: copied(&item.policy_sha256, 64, ledger)?,
        reviewer_key: copied(options.reviewer_key, 128, ledger)?,
        reviewer_role: copied(options.reviewer_role, 128, ledger)?,
        disposition: options.disposition,
        responded_at: copied(options.responded_at, 20, ledger)?,
        rationale: copied(rationale, 8192, ledger)?,
        abstention_reason: options
            .abstention_reason
            .map(|reason| copied(reason, 128, ledger))
            .transpose()?,
        proposed_edit: (),
        supersedes,
    };
    validate::response(&response, ledger, control)?;
    Ok(response)
}

#[cfg(test)]
#[path = "commands_tests.rs"]
/// Pure private-request/selection refusals only; genuine five-flow controls remain Root-owned.
mod tests;

#[cfg(all(test, unix))]
#[path = "commands_applicability_tests.rs"]
/// Actual synthetic applicability command/native proof controls; no human or platform acceptance.
mod applicability_command_tests;
