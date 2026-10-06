//! Strict inert two-route locator on the original caller's admission and control.
//! Its paths cannot establish native provenance, capture membership or currentness.
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase, structural};
use super::wire_v3::AuthoringPlanInputsV1;
use crate::workspace::preparation::WorkControl;
use std::path::Path;
/// Fixed closed locator schema, independent of Queue/3 or Lifecycle/2 envelopes.
const SCHEMA: &str = include_str!("../../schemas/forge.review-authoring-plan-inputs-1.schema.json");
/// Decode complete captured bytes; typed growth is admitted by the actual shared strict primitive.
pub(super) fn decode(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<AuthoringPlanInputsV1, ContractError> {
    phase(ledger, control, |ledger, control| {
        let value = structural::admitted_value(raw, 1024 * 1024, SCHEMA, ledger, control)?;
        ledger.bytes(raw.len())?;
        let ordinary = serde_json::from_value(value).map_err(|_| ContractError::Invalid);
        checkpoint(ledger, control)?;
        let locator: AuthoringPlanInputsV1 = ordinary?;
        for route in [&locator.project.path, &locator.plan.path] {
            checkpoint(ledger, control)?;
            ledger.bytes(route.len())?;
            let path = Path::new(route);
            let count = path.components().count();
            ledger.visits(count + 1)?;
            if route.len() > 4096 || count > 64 {
                return Err(ContractError::Invalid);
            }
            crate::linkage::fresh::validate_relative(path).map_err(|_| ContractError::Invalid)?;
        }
        Ok(locator)
    })
}
