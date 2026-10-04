//! Crate-private original capture and before-growth budgets for evidence inspection.
//!
//! Original bytes are retained once per exact compatible path. Parsed forms and
//! fixed scratch buffers have separate input bounds; this is not a heap ceiling.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::Serialize;

use crate::ForgeError;
use crate::linkage::fresh::{self, AbsenceGeneration, FileGeneration, RootGeneration};

/// Complete retained original limit across both evidence inspection lanes.
pub(crate) const MAX_ORIGINAL_BYTES: usize = 100 * 1024 * 1024;
/// Complete physical present and absent observation bound, including declarations.
pub(crate) const MAX_INPUT_SLOTS: usize = 10_137;
/// Complete repeated native inventory, link, evidence and assertion relationship bound.
pub(crate) const MAX_RELATIONSHIPS: usize = 100_000;
/// Complete escaped JSON projection and public report bound.
pub(crate) const MAX_PROJECTION_BYTES: usize = 10 * 1024 * 1024;

/// Compatible native models and declarations sharing an exact captured original.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureRole {
    /// POA&M authoring declaration; it never grants native terminal admission.
    WorkflowDeclaration,
    /// Actual empty foundation declaration for explicit reviewed-risk authoring.
    RiskScaffoldDeclaration,
    /// Complete caller-authored reviewed-risk request; it authenticates no reviewer.
    RiskAuthoringRequest,
    /// Explicit read-only assertion-to-link companion declaration.
    EvidenceBindings,
    /// Explicit linkage manifest pinned by the companion declaration.
    LinkageManifest,
    /// Original supported Assessment Results source.
    AssessmentResults,
    /// Original supported Assessment Plan source.
    AssessmentPlan,
    /// SSP source or exactly compatible linkage implementation resource.
    SystemSecurityPlan,
    /// Profile source or exactly compatible linkage requirement resource.
    Profile,
    /// Catalog source or exactly compatible linkage requirement/companion resource.
    Catalog,
    /// Native Component Definition linkage implementation resource.
    ComponentDefinition,
    /// Local evidence, which must not alias any source or declaration role.
    LocalEvidence,
}

/// Private counters shared only by actual original holders in one preparation.
struct Ledger {
    /// Actual retained original byte allocations, released with their owner.
    original_bytes: Cell<usize>,
    /// Monotonic complete physical observation slots admitted before insertion.
    input_slots: Cell<usize>,
    /// Monotonic complete relationships admitted before insertion or filtering.
    relationships: Cell<usize>,
}

/// Sole owner of shared before-growth admission for one complete inspection.
pub(crate) struct ReadBudget {
    /// Shared private lifetime ledger; callers cannot release another holder.
    ledger: Rc<Ledger>,
}

impl ReadBudget {
    /// Start the fixed combined inspection budget without changing legacy defaults.
    fn new() -> Self {
        Self {
            ledger: Rc::new(Ledger {
                original_bytes: Cell::new(0),
                input_slots: Cell::new(0),
                relationships: Cell::new(0),
            }),
        }
    }

    /// Limit the next actual retained original before its confined reader grows a buffer.
    fn remaining_read(&self, per_file: u64) -> Result<u64, ForgeError> {
        let remaining = MAX_ORIGINAL_BYTES
            .checked_sub(self.ledger.original_bytes.get())
            .ok_or_else(|| error("original byte accounting is invalid"))?;
        Ok(per_file.min(
            u64::try_from(remaining).map_err(|_| error("original byte limit conversion failed"))?,
        ))
    }

    /// Reserve one complete observation before opening or storing it; errors abort preparation.
    fn input_slot(&mut self) -> Result<(), ForgeError> {
        let count = self
            .ledger
            .input_slots
            .get()
            .checked_add(1)
            .filter(|count| *count <= MAX_INPUT_SLOTS)
            .ok_or_else(|| error("complete evidence input observation limit exceeded"))?;
        self.ledger.input_slots.set(count);
        Ok(())
    }

    /// Attach original raw bytes to their actual allocation owner after a pre-limited read.
    fn retain(&mut self, length: usize) -> Result<RawCharge, ForgeError> {
        let total = self
            .ledger
            .original_bytes
            .get()
            .checked_add(length)
            .filter(|total| *total <= MAX_ORIGINAL_BYTES)
            .ok_or_else(|| error("complete retained original byte limit exceeded"))?;
        self.ledger.original_bytes.set(total);
        Ok(RawCharge { ledger: Rc::clone(&self.ledger), bytes: length })
    }

    /// Admit every complete repeated relationship before any corresponding vector/map growth.
    fn relationships(&mut self, count: usize) -> Result<(), ForgeError> {
        let total = self
            .ledger
            .relationships
            .get()
            .checked_add(count)
            .filter(|total| *total <= MAX_RELATIONSHIPS)
            .ok_or_else(|| error("complete evidence relationship limit exceeded"))?;
        self.ledger.relationships.set(total);
        Ok(())
    }
}

/// Byte charge released only after its complete original allocation has been dropped.
struct RawCharge {
    /// Ledger shared with the sole preparation and all still-held originals.
    ledger: Rc<Ledger>,
    /// Exact length of this actual retained original allocation.
    bytes: usize,
}

impl Drop for RawCharge {
    /// Release only this owner's exact admitted allocation after the bytes field drops.
    fn drop(&mut self) {
        self.ledger.original_bytes.set(self.ledger.original_bytes.get() - self.bytes);
    }
}

/// Original allocation and private actual confinement proof; no detached public construction.
struct Original {
    /// Complete original bytes, declared before charge so actual bytes drop first.
    bytes: Vec<u8>,
    /// Actual root-relative held-handle generation and ancestor observations.
    generation: FileGeneration,
    /// Live original allocation admission, dropped after bytes and generation.
    _charge: RawCharge,
}

/// Sealed shared reference to one actual original allocation, without copying raw bytes.
#[derive(Clone)]
pub(crate) struct CaptureLease {
    /// Shared allocation owner; clone cannot create a second uncharged Vec.
    original: Rc<Original>,
}

impl CaptureLease {
    /// Borrow the complete captured original generation without re-reading a path.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.original.bytes
    }

    /// Borrow the normalized root descendant for internal joins and collision checks.
    pub(crate) fn path(&self) -> &Path {
        self.original.generation.relative()
    }

    /// Return only the actual platform file identity used by internal alias checks.
    pub(crate) fn identity(&self) -> (u64, u64) {
        self.original.generation.identity()
    }

    /// Stream-reconcile exact original bytes, identity and safe ancestry without a full duplicate.
    pub(crate) fn verify_inputs(&self) -> Result<(), ForgeError> {
        fresh::verify_file(&self.original.generation, &self.original.bytes)
    }
}

/// Sealed actual local absence observation with a qualified root and existing ancestors.
#[derive(Clone)]
pub(crate) struct AbsenceLease {
    /// Actual observation, never constructed from an IO error string or caller label.
    generation: Rc<AbsenceGeneration>,
}

/// Present exact original or typed unavailable-local observation; other faults refuse.
#[derive(Clone)]
pub(crate) enum LocalObservation {
    /// Original bytes and exact identity were actually captured.
    Present(CaptureLease),
    /// The exact first missing component was observed under qualified ancestors.
    Absent(AbsenceLease),
}

/// Complete private original registry, with one raw/input/relationship pool for both lanes.
pub(crate) struct CaptureSession {
    /// Qualified normalized actual root; no caller-authored native identity.
    root: Rc<RootGeneration>,
    /// Exact path-to-role and original/absence registry, without relative-base guessing.
    inputs: BTreeMap<PathBuf, (CaptureRole, LocalObservation)>,
    /// Every actual declared evidence root, including unused roots, bounded by the existing 64.
    directories: BTreeMap<PathBuf, Rc<RootGeneration>>,
    /// Non-observation output namespace guard reserved before any file or declared-directory capture.
    reserved_output: Option<PathBuf>,
    /// Shared complete budget; not separately reset by source and linkage consumers.
    budget: ReadBudget,
}

impl CaptureSession {
    /// Qualify one actual directory root before any source or evidence observation.
    pub(crate) fn new(root: &Path) -> Result<Self, ForgeError> {
        Ok(Self {
            root: Rc::new(fresh::qualify_root(root)?),
            inputs: BTreeMap::new(),
            directories: BTreeMap::new(),
            reserved_output: None,
            budget: ReadBudget::new(),
        })
    }

    /// Borrow the qualified root only for actual source-relative base calculation.
    pub(crate) fn root(&self) -> &Path {
        self.root.path()
    }

    /// Borrow an existing same-role present original without IO, detached bytes or new admission.
    pub(crate) fn captured(&self, relative: &Path, role: CaptureRole) -> Result<&[u8], ForgeError> {
        fresh::validate_relative(relative)?;
        match self.inputs.get(relative) {
            Some((previous_role, LocalObservation::Present(lease))) if *previous_role == role => {
                Ok(lease.bytes())
            }
            _ => Err(error("sealed captured original is absent, unrecorded or incompatible")),
        }
    }

    /// Reserve one normalized report target without IO before any original or declared-directory observation.
    pub(crate) fn reserve_output(&mut self, relative: &Path) -> Result<(), ForgeError> {
        fresh::validate_relative(relative)?;
        if !self.inputs.is_empty() || !self.directories.is_empty() {
            return Err(error("output namespace must be reserved before input capture"));
        }
        if let Some(previous) = &self.reserved_output {
            if previous != relative {
                return Err(error("output namespace was already reserved"));
            }
        } else {
            self.reserved_output = Some(relative.to_path_buf());
        }
        Ok(())
    }

    /// Capture one required source/declaration or share an exact compatible actual generation.
    pub(crate) fn required(
        &mut self,
        relative: &Path,
        role: CaptureRole,
        max_bytes: u64,
    ) -> Result<CaptureLease, ForgeError> {
        match self.observe(relative, role, max_bytes, false)? {
            LocalObservation::Present(lease) => Ok(lease),
            LocalObservation::Absent(_) => {
                Err(error("required evidence inspection input is unavailable"))
            }
        }
    }

    /// Capture actual local evidence or typed missing ancestry; never downgrade unsafe/unreadable faults.
    pub(crate) fn optional_local(
        &mut self,
        relative: &Path,
        max_bytes: u64,
    ) -> Result<LocalObservation, ForgeError> {
        self.observe(relative, CaptureRole::LocalEvidence, max_bytes, true)
    }

    /// Admit a complete native/reference/assertion relationship before insertion or filtering.
    pub(crate) fn relationships(&mut self, count: usize) -> Result<(), ForgeError> {
        self.budget.relationships(count)
    }

    /// Retain an actual declared directory generation, including unused roots, before exposing freshness metadata.
    pub(crate) fn directory(&mut self, relative: &Path) -> Result<(), ForgeError> {
        fresh::validate_relative(relative)?;
        check_output_namespace(self.reserved_output.as_deref(), relative, true)?;
        if let Some(original) = self.directories.get(relative) {
            return fresh::verify_root(original);
        }
        if self.directories.len() >= crate::linkage::manifest::MAX_EVIDENCE_ROOTS {
            return Err(error("complete declared evidence root limit exceeded"));
        }
        self.relationships(1)?;
        fresh::verify_root(&self.root)?;
        let generation = fresh::qualify_root(&self.root.path().join(relative))?;
        if self.directories.values().any(|previous| previous.identity() == generation.identity()) {
            return Err(error("declared evidence directories alias an actual root identity"));
        }
        fresh::verify_root(&self.root)?;
        self.directories.insert(relative.to_path_buf(), Rc::new(generation));
        Ok(())
    }

    /// Validate, capture and charge one complete observation without another lane resetting admission.
    fn observe(
        &mut self,
        relative: &Path,
        role: CaptureRole,
        max_bytes: u64,
        allow_missing: bool,
    ) -> Result<LocalObservation, ForgeError> {
        fresh::validate_relative(relative)?;
        check_output_namespace(self.reserved_output.as_deref(), relative, false)?;
        if let Some((previous_role, observation)) = self.inputs.get(relative) {
            if *previous_role != role {
                return Err(error("evidence inspection input has incompatible roles"));
            }
            match observation {
                LocalObservation::Present(lease) => {
                    if u64::try_from(lease.bytes().len())
                        .map_err(|_| error("input size conversion failed"))?
                        > max_bytes
                    {
                        return Err(error("shared original exceeds this role's input limit"));
                    }
                    lease.verify_inputs()?;
                }
                LocalObservation::Absent(lease) => {
                    if !allow_missing {
                        return Err(error("required evidence inspection input is unavailable"));
                    }
                    fresh::verify_absence(&lease.generation)?;
                }
            }
            return Ok(observation.clone());
        }
        self.budget.input_slot()?;
        let limit = self.budget.remaining_read(max_bytes)?;
        let observation =
            match fresh::capture_local(Rc::clone(&self.root), relative, limit, allow_missing)? {
                fresh::CapturedLocal::Present(bytes, generation) => {
                    if self.inputs.values().any(|(_, previous)| {
                        matches!(previous,
                    LocalObservation::Present(lease) if lease.identity() == generation.identity())
                    }) {
                        return Err(error(
                            "distinct evidence inspection paths alias an actual file identity",
                        ));
                    }
                    let charge = self.budget.retain(bytes.len())?;
                    LocalObservation::Present(CaptureLease {
                        original: Rc::new(Original { bytes, generation, _charge: charge }),
                    })
                }
                fresh::CapturedLocal::Absent(generation) => {
                    LocalObservation::Absent(AbsenceLease { generation: Rc::new(generation) })
                }
            };
        self.inputs.insert(relative.to_path_buf(), (role, observation.clone()));
        Ok(observation)
    }

    /// Move actual complete originals into the private proof; no authorizing capability is created.
    pub(crate) fn finish(self) -> CaptureProof {
        CaptureProof {
            root: self.root,
            inputs: self.inputs,
            directories: self.directories,
            reserved_output: self.reserved_output,
        }
    }
}

/// Complete actual present/absent generations consumed by the public inspection holder.
pub(crate) struct CaptureProof {
    /// Actual qualified root and safe root ancestor generations.
    root: Rc<RootGeneration>,
    /// All actual captured originals and typed local absence observations.
    inputs: BTreeMap<PathBuf, (CaptureRole, LocalObservation)>,
    /// Actual complete declared evidence roots, including roots with no referenced local files.
    directories: BTreeMap<PathBuf, Rc<RootGeneration>>,
    /// Same immutable output namespace guard; it grants neither original nor output authority.
    reserved_output: Option<PathBuf>,
}

impl CaptureProof {
    /// Count distinct actually retained present originals, excluding unavailable-local observations.
    pub(crate) fn captured_original_generations(&self) -> usize {
        self.inputs
            .values()
            .filter(|(_, observation)| matches!(observation, LocalObservation::Present(_)))
            .count()
    }

    /// Reconcile actual roots, all present original bytes and exact absent components before publication.
    pub(crate) fn verify_inputs(&self) -> Result<(), ForgeError> {
        for relative in self.inputs.keys() {
            check_output_namespace(self.reserved_output.as_deref(), relative, false)?;
        }
        for relative in self.directories.keys() {
            check_output_namespace(self.reserved_output.as_deref(), relative, true)?;
        }
        fresh::verify_root(&self.root)?;
        for directory in self.directories.values() {
            fresh::verify_root(directory)?;
        }
        for (_, observation) in self.inputs.values() {
            match observation {
                LocalObservation::Present(lease) => lease.verify_inputs()?,
                LocalObservation::Absent(lease) => fresh::verify_absence(&lease.generation)?,
            }
        }
        fresh::verify_root(&self.root)
    }

    /// Borrow complete actual present and missing paths for Root's output collision refusal.
    pub(crate) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.inputs
            .keys()
            .chain(self.directories.keys())
            .map(|relative| self.root.path().join(relative))
    }
}

/// Compare lossless component spellings using ASCII folding without lossy Unicode/path normalization.
fn folded_prefix(prefix: &Path, full: &Path) -> bool {
    let mut complete = full.components();
    for component in prefix.components() {
        let Some(other) = complete.next() else {
            return false;
        };
        let left = component.as_os_str().as_encoded_bytes();
        let right = other.as_os_str().as_encoded_bytes();
        if left.len() != right.len()
            || !left.iter().zip(right).all(|(a, b)| a.eq_ignore_ascii_case(b))
        {
            return false;
        }
    }
    true
}

/// Refuse output aliases/ancestors before IO; declared directories permit unrelated descendant reports.
fn check_output_namespace(
    output: Option<&Path>,
    input: &Path,
    directory: bool,
) -> Result<(), ForgeError> {
    if let Some(output) = output {
        if folded_prefix(output, input) || (!directory && folded_prefix(input, output)) {
            return Err(error("report output aliases or overlaps a held input namespace"));
        }
    }
    Ok(())
}

/// Bounded escaped row admission BEFORE metadata vector cloning, with complete final encoding.
pub(crate) struct ProjectionBudget {
    /// Complete compact row bytes and conservative separators already admitted.
    admitted: usize,
}

impl ProjectionBudget {
    /// Start one independent complete 10 MiB projection, not another source byte pool.
    pub(crate) fn new() -> Self {
        Self { admitted: 0 }
    }

    /// Count serialized borrowed fields and a separator before retaining this complete metadata row.
    pub(crate) fn admit_row<T: Serialize>(&mut self, row: &T) -> Result<(), ForgeError> {
        let remaining = MAX_PROJECTION_BYTES
            .checked_sub(self.admitted)
            .ok_or_else(|| error("projection accounting is invalid"))?;
        let mut writer = CountingWriter { remaining, written: 0 };
        serde_json::to_writer(&mut writer, row)
            .map_err(|_| error("complete evidence metadata projection exceeds its bound"))?;
        let total = self
            .admitted
            .checked_add(writer.written)
            .and_then(|count| count.checked_add(1))
            .filter(|count| *count <= MAX_PROJECTION_BYTES)
            .ok_or_else(|| error("complete evidence metadata projection exceeds its bound"))?;
        self.admitted = total;
        Ok(())
    }

    /// Encode complete JSON through a bounded writer; refuse instead of returning a valid prefix.
    pub(crate) fn encode<T: Serialize>(report: &T) -> Result<Vec<u8>, ForgeError> {
        let mut writer = BoundedWriter { bytes: Vec::new() };
        serde_json::to_writer_pretty(&mut writer, report)
            .map_err(|_| error("complete evidence report exceeds its bound"))?;
        writer.write_all(b"\n").map_err(|_| error("complete evidence report exceeds its bound"))?;
        Ok(writer.bytes)
    }
}

/// Nonretaining escaped JSON measurement that refuses before exceeding remaining metadata admission.
struct CountingWriter {
    /// Remaining complete row admission before this row grows any retained array.
    remaining: usize,
    /// Actual escaped JSON bytes observed by serde, not string-length estimates.
    written: usize,
}

impl io::Write for CountingWriter {
    /// Count one serializer fragment before admitting it, with checked arithmetic.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let total = self
            .written
            .checked_add(bytes.len())
            .filter(|count| *count <= self.remaining)
            .ok_or_else(|| io::Error::other("metadata bound"))?;
        self.written = total;
        Ok(bytes.len())
    }

    /// Complete the no-allocation counting sink without external IO.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Private complete JSON byte sink checking the bound before every retained buffer growth.
struct BoundedWriter {
    /// Complete encoded output kept private until all serialization succeeds.
    bytes: Vec<u8>,
}

impl io::Write for BoundedWriter {
    /// Refuse before appending any fragment that would exceed the complete output cap.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes
            .len()
            .checked_add(bytes.len())
            .filter(|count| *count <= MAX_PROJECTION_BYTES)
            .ok_or_else(|| io::Error::other("report bound"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    /// Complete the private output sink without external IO.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Return a fixed redacted internal failure; public inspection never prints raw original paths/content.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(reason.to_string())
}

/// Proposed source controls for actual capture/native projection; no execution is implied.
#[cfg(test)]
mod tests {
    use super::*;

    /// Create a genuine private directory and one known original for proposed native reader controls.
    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("original.bin"), b"original").unwrap();
        (directory, root)
    }

    /// Repeated exact same-role paths share the actual allocation and consume one present slot.
    #[test]
    fn same_path_and_role_share_one_original_allocation() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        let first = capture.required(Path::new("original.bin"), CaptureRole::Catalog, 100).unwrap();
        let second =
            capture.required(Path::new("original.bin"), CaptureRole::Catalog, 100).unwrap();
        assert!(Rc::ptr_eq(&first.original, &second.original));
        assert_eq!(capture.budget.ledger.original_bytes.get(), 8);
        assert_eq!(capture.budget.ledger.input_slots.get(), 1);
        assert_eq!(capture.finish().captured_original_generations(), 1);
    }

    /// A local evidence claim cannot reuse an original captured under an incompatible native source role.
    #[test]
    fn same_path_different_role_refuses_without_another_observation() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.required(Path::new("original.bin"), CaptureRole::AssessmentResults, 100).unwrap();
        assert!(capture.optional_local(Path::new("original.bin"), 100).is_err());
        assert_eq!(capture.budget.ledger.input_slots.get(), 1);
        assert_eq!(capture.budget.ledger.original_bytes.get(), 8);
    }

    /// A repeated compatible role with a smaller per-input bound cannot bypass that requested bound.
    #[test]
    fn shared_lease_still_obeys_smaller_requested_limit() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.required(Path::new("original.bin"), CaptureRole::Catalog, 100).unwrap();
        assert!(capture.required(Path::new("original.bin"), CaptureRole::Catalog, 7).is_err());
    }

    /// A no-IO parser borrow shares the actual allocation and refuses missing or incompatible proof claims.
    #[test]
    fn parser_lookup_borrows_only_existing_compatible_present_originals() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        assert!(capture.captured(Path::new("original.bin"), CaptureRole::Catalog).is_err());
        let lease = capture.required(Path::new("original.bin"), CaptureRole::Catalog, 8).unwrap();
        let captured = capture.captured(Path::new("original.bin"), CaptureRole::Catalog).unwrap();
        assert_eq!(captured.as_ptr(), lease.bytes().as_ptr());
        assert_eq!(captured.len(), lease.bytes().len());
        assert!(capture.captured(Path::new("original.bin"), CaptureRole::LocalEvidence).is_err());
        let _missing = capture.optional_local(Path::new("missing.bin"), 8).unwrap();
        assert!(capture.captured(Path::new("missing.bin"), CaptureRole::LocalEvidence).is_err());
        assert_eq!(capture.finish().captured_original_generations(), 1);
    }

    /// Folded present-file aliases refuse before observation slots, bytes or native reads are admitted.
    #[test]
    fn reserved_report_alias_refuses_before_present_file_capture() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.reserve_output(Path::new("ORIGINAL.bin")).unwrap();
        assert!(capture.required(Path::new("original.bin"), CaptureRole::Catalog, 100).is_err());
        assert_eq!(capture.budget.ledger.input_slots.get(), 0);
        assert_eq!(capture.budget.ledger.original_bytes.get(), 0);
        assert!(capture.finish().input_paths().next().is_none());
    }

    /// Publication cannot create a first-missing ancestor or use an unavailable dependency as its ancestor.
    #[test]
    fn reserved_report_overlaps_missing_file_ancestors_before_observation() {
        let (_directory, root) = fixture();
        for (report, input) in
            [("MISSING", "missing/file.bin"), ("missing/file.bin/report", "MISSING/file.bin")]
        {
            let mut capture = CaptureSession::new(&root).unwrap();
            capture.reserve_output(Path::new(report)).unwrap();
            assert!(capture.optional_local(Path::new(input), 100).is_err());
            assert_eq!(capture.budget.ledger.input_slots.get(), 0);
            assert!(capture.inputs.is_empty());
        }
    }

    /// Declared directories refuse output aliases/ancestors while permitting unrelated reports beneath them.
    #[test]
    fn declared_directory_guard_does_not_blanket_refuse_report_descendants() {
        let (_directory, root) = fixture();
        std::fs::create_dir(root.join("evidence")).unwrap();
        let mut permitted = CaptureSession::new(&root).unwrap();
        permitted.reserve_output(Path::new("evidence/report.json")).unwrap();
        permitted.directory(Path::new("evidence")).unwrap();
        assert_eq!(permitted.finish().captured_original_generations(), 0);
        let mut alias = CaptureSession::new(&root).unwrap();
        alias.reserve_output(Path::new("EVIDENCE")).unwrap();
        assert!(alias.directory(Path::new("evidence")).is_err());
        assert!(alias.directories.is_empty());
        let mut ancestor = CaptureSession::new(&root).unwrap();
        ancestor.reserve_output(Path::new("EVIDENCE")).unwrap();
        assert!(ancestor.directory(Path::new("evidence/unused")).is_err());
        assert!(ancestor.directories.is_empty());
    }

    /// A late report reservation cannot retroactively claim preflight of already captured inputs.
    #[test]
    fn output_reservation_must_precede_all_observations() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.required(Path::new("original.bin"), CaptureRole::Catalog, 100).unwrap();
        assert!(capture.reserve_output(Path::new("report.json")).is_err());
        assert!(capture.reserved_output.is_none());
    }

    /// Complete original charge is retained until both proof and every shared actual lease drop.
    #[test]
    fn raw_credit_is_not_released_while_any_original_holder_lives() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        let ledger = Rc::clone(&capture.budget.ledger);
        let lease =
            capture.required(Path::new("original.bin"), CaptureRole::LocalEvidence, 100).unwrap();
        let proof = capture.finish();
        drop(proof);
        assert_eq!(ledger.original_bytes.get(), 8);
        drop(lease);
        assert_eq!(ledger.original_bytes.get(), 0);
    }

    /// Remaining original admission reaches the actual confined reader before any oversized original retention.
    #[test]
    fn remaining_budget_refuses_actual_file_before_retention() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.budget.ledger.original_bytes.set(MAX_ORIGINAL_BYTES - 7);
        assert!(
            capture.required(Path::new("original.bin"), CaptureRole::LocalEvidence, 100).is_err()
        );
        assert!(capture.inputs.is_empty());
        assert_eq!(capture.budget.ledger.original_bytes.get(), MAX_ORIGINAL_BYTES - 7);
    }

    /// Complete input slot refusal occurs before inserting any present or unavailable observation.
    #[test]
    fn complete_input_slot_limit_refuses_without_capture() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.budget.ledger.input_slots.set(MAX_INPUT_SLOTS);
        assert!(capture.optional_local(Path::new("missing.bin"), 100).is_err());
        assert!(capture.inputs.is_empty());
        assert_eq!(capture.budget.ledger.input_slots.get(), MAX_INPUT_SLOTS);
    }

    /// Every repeated native/reference row shares one monotonic complete counter and overflow leaves it unchanged.
    #[test]
    fn complete_relationship_limit_and_overflow_refuse_before_growth() {
        let mut budget = ReadBudget::new();
        budget.relationships(MAX_RELATIONSHIPS).unwrap();
        assert!(budget.relationships(1).is_err());
        assert!(budget.relationships(usize::MAX).is_err());
        assert_eq!(budget.ledger.relationships.get(), MAX_RELATIONSHIPS);
    }

    /// Actual absence consumes a slot but never a present-original or raw-byte count.
    #[test]
    fn unavailable_observation_is_not_zero_byte_present_evidence() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        assert!(matches!(
            capture.optional_local(Path::new("missing.bin"), 100).unwrap(),
            LocalObservation::Absent(_)
        ));
        assert_eq!(capture.budget.ledger.input_slots.get(), 1);
        assert_eq!(capture.budget.ledger.original_bytes.get(), 0);
        let proof = capture.finish();
        assert_eq!(proof.captured_original_generations(), 0);
        proof.verify_inputs().unwrap();
        assert_eq!(proof.input_paths().collect::<Vec<_>>(), vec![root.join("missing.bin")]);
    }

    /// An actual safe file appearing after the unavailable observation invalidates publication proof.
    #[test]
    fn unavailable_file_appearance_refuses_current_proof() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.optional_local(Path::new("missing.bin"), 100).unwrap();
        let proof = capture.finish();
        std::fs::write(root.join("missing.bin"), b"appeared").unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// A newly appeared parent changes the exact first missing component even when the leaf is still absent.
    #[test]
    fn unavailable_parent_appearance_refuses_exact_absence_generation() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.optional_local(Path::new("missing/leaf.bin"), 100).unwrap();
        let proof = capture.finish();
        std::fs::create_dir(root.join("missing")).unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// Nonregular local evidence is invalid rather than being relabeled as unavailable.
    #[test]
    fn directory_final_input_is_invalid_not_missing() {
        let (_directory, root) = fixture();
        std::fs::create_dir(root.join("not-file")).unwrap();
        let mut capture = CaptureSession::new(&root).unwrap();
        assert!(capture.optional_local(Path::new("not-file"), 100).is_err());
        assert!(capture.inputs.is_empty());
    }

    /// Exact current bytes are streamed beyond one scratch block and changed tail bytes refuse.
    #[test]
    fn streamed_recheck_compares_complete_tail_and_eof() {
        let (_directory, root) = fixture();
        let mut raw = vec![b'a'; 2 * 32 * 1024 + 7];
        std::fs::write(root.join("large.bin"), &raw).unwrap();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture
            .required(Path::new("large.bin"), CaptureRole::LocalEvidence, raw.len() as u64)
            .unwrap();
        let proof = capture.finish();
        proof.verify_inputs().unwrap();
        *raw.last_mut().unwrap() = b'b';
        std::fs::write(root.join("large.bin"), &raw).unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// An unused declared evidence directory is retained/rechecked independently of present file counts.
    #[test]
    fn unused_directory_is_a_dependency_but_not_a_present_original() {
        let (_directory, root) = fixture();
        std::fs::create_dir(root.join("evidence")).unwrap();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.directory(Path::new("evidence")).unwrap();
        assert_eq!(capture.budget.ledger.relationships.get(), 1);
        let proof = capture.finish();
        assert_eq!(proof.captured_original_generations(), 0);
        assert_eq!(proof.input_paths().collect::<Vec<_>>(), vec![root.join("evidence")]);
        proof.verify_inputs().unwrap();
    }

    /// Native same-byte replacement is rejected by actual identity while the original instance remains held.
    #[cfg(unix)]
    #[test]
    fn same_bytes_different_instance_refuses_on_unix() {
        let (_directory, root) = fixture();
        let mut capture = CaptureSession::new(&root).unwrap();
        capture.required(Path::new("original.bin"), CaptureRole::LocalEvidence, 100).unwrap();
        let proof = capture.finish();
        std::fs::write(root.join("new.bin"), b"original").unwrap();
        std::fs::rename(root.join("new.bin"), root.join("original.bin")).unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// Escaping is counted before retained row admission rather than measuring only logical string lengths.
    #[test]
    fn escaped_projection_bound_refuses_before_second_row_admission() {
        let raw = "\0".repeat(1024 * 1024);
        let mut projection = ProjectionBudget::new();
        projection.admit_row(&raw).unwrap();
        let before = projection.admitted;
        assert!(projection.admit_row(&raw).is_err());
        assert_eq!(projection.admitted, before);
        assert!(before > raw.len());
    }

    /// Final complete pretty encoding refuses oversized output without returning a truncated successful prefix.
    #[test]
    fn final_encoding_is_complete_or_refused() {
        assert_eq!(ProjectionBudget::encode(&["safe"]).unwrap(), b"[\n  \"safe\"\n]\n");
        let oversized = "x".repeat(MAX_PROJECTION_BYTES);
        assert!(ProjectionBudget::encode(&oversized).is_err());
    }
}
