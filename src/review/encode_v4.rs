//! Finite separately closed edit-capable Response/4 and Dispositions/4 output/readback.
//! Queues remain the closed /3 shape and are encoded by the unchanged /3 encoder.
//! No encoded byte vector supplies native ownership or publication authority.

/// Separately closed finite Response/4 and recorded Dispositions/4 output.
#[path = "encode_v4_operational.rs"]
mod operational;
#[cfg(test)]
pub(crate) use operational::readback_response;
pub(crate) use operational::{
    compare_dispositions, compare_responses, dispositions, dispositions_operands, response,
    response_operands,
};

#[cfg(test)]
#[path = "encode_v4_tests.rs"]
mod tests;
