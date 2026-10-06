//! Plain admitted review utilities on the unchanged complete authoring evaluator.
//! No result confers capture, physical membership, currentness or reviewer authority.

use super::{Admission, AuthoringCharge, AuthoringPlan, BorrowedAuthoringError};
use crate::workspace::preparation::WorkControl;
use std::path::{Path, PathBuf};

/// Resolve the complete native project-relative dependency using the actual maintained rule.
/// The returned label is ordinary data; its real capture geometry remains receiver-owned.
pub(crate) fn resolve_review_dependency<E>(
    base: &Path,
    relative: &Path,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(AuthoringCharge) -> Result<(), E>,
) -> Result<PathBuf, BorrowedAuthoringError<E>> {
    let mut a = Admission { control, admit, _error: std::marker::PhantomData };
    a.step()?;
    let ordinary = (|| {
        super::admit_route(base, relative, &mut a)?;
        a.native(|| super::super::input::contained_dependency(base, relative))
    })();
    a.finish(ordinary)
}

/// Compare every native plan field and ordered private operand against the stored original.
/// Only JSON object key order and insignificant syntax whitespace may differ.
/// Empty malformed input is ordinary Domain; actual bounds and callback stops stay sticky.
pub(crate) fn compare_review_stored_plan<E>(
    raw: &[u8],
    plan: &AuthoringPlan,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(AuthoringCharge) -> Result<(), E>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let mut a = Admission { control, admit, _error: std::marker::PhantomData };
    a.step()?;
    let ordinary = (|| {
        if raw.len() > 10 * 1024 * 1024 {
            return a.capacity();
        }
        let stored = super::strict(raw, "stored authoring plan", &mut a)?;
        let metric = a.measure(plan)?;
        a.materialize(metric, 1, 2)?;
        let expected = a.native(|| {
            serde_json::to_value(plan).map_err(|cause| {
                super::super::error(format!("cannot serialize complete plan: {cause}"))
            })
        })?;
        let actual = a.measure(&stored)?;
        let nodes = a.add(metric.nodes, actual.nodes)?;
        let bytes = a.add(metric.encoded, actual.encoded)?;
        a.work(nodes, bytes, nodes)?;
        let equal = expected == stored;
        a.step()?;
        if !equal {
            return super::domain("stored plan differs from the entire regenerated native plan");
        }
        Ok(())
    })();
    a.finish(ordinary)
}

/// Observe original root text from data already validated by the complete native evaluator.
/// This ordinary projection independently checks shape/UUID syntax but cannot attest native validation.
pub(crate) fn review_root_text<E>(
    raw: &[u8],
    model: &str,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(AuthoringCharge) -> Result<(), E>,
) -> Result<String, BorrowedAuthoringError<E>> {
    let mut a = Admission { control, admit, _error: std::marker::PhantomData };
    a.step()?;
    let ordinary = (|| {
        if raw.len() > 10 * 1024 * 1024 {
            return a.capacity();
        }
        a.work(1, model.len(), 1)?;
        if !matches!(model, "catalog" | "profile" | "mapping-collection") {
            return super::domain("unsupported native authoring root projection");
        }
        let value = super::strict(raw, "validated native root projection", &mut a)?;
        let tree = a.measure(&value)?;
        a.work(tree.nodes, tree.encoded, tree.nodes)?;
        let object = value.as_object().ok_or_else(|| {
            BorrowedAuthoringError::Domain(super::super::error(
                "native root projection requires an object",
            ))
        })?;
        let lookup = a.mul(model.len(), object.len())?;
        a.work(object.len(), lookup, object.len())?;
        if object.len() != 1 {
            return super::domain("native root projection requires exactly one model");
        }
        let root = object
            .get(model)
            .and_then(|v| v.get("uuid"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                BorrowedAuthoringError::Domain(super::super::error("native root UUID absent"))
            })?;
        a.work(1, root.len(), 1)?;
        let extent = a.add(root.len(), std::mem::size_of::<String>())?;
        a.reserve(extent)?;
        a.native(|| {
            uuid::Uuid::parse_str(root).map_err(|_| super::super::error("native root UUID invalid"))
        })?;
        Ok(root.to_owned())
    })();
    a.finish(ordinary)
}
