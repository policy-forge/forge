//! Local workspace services and adapters. Project files remain authoritative.

pub(crate) mod actions;
pub(crate) mod assets;
pub(crate) mod bundle_effects;
pub(crate) mod bundles;
pub(crate) mod contract;
pub(crate) mod domain;
pub(crate) mod effects;
pub(crate) mod http;
pub(crate) mod impact;
pub(crate) mod index;
pub(crate) mod inspection;
pub(crate) mod lifecycle;
pub(crate) mod preparation;
pub(crate) mod provenance;
pub(crate) mod reports;
pub(crate) mod root;
pub(crate) mod services;
pub(crate) mod session;

pub(crate) mod source_bundle_effects;
pub(crate) mod source_bundles;
pub(crate) mod source_stream_reads;
pub(crate) mod source_transfers;
pub(crate) mod source_validation;
pub(crate) mod staged_source_bundles;

/// Select a supported major before project capture, prompts, credentials or listening.
pub(crate) fn launch(
    project: &std::path::Path,
    read_only: bool,
    machine: bool,
    no_open: bool,
    api_major: u8,
) -> Result<(), crate::ForgeError> {
    let major = match api_major {
        1 => contract::ApiMajor::V1,
        2 => contract::ApiMajor::V2,
        _ => {
            return Err(crate::ForgeError::InvalidArgument(contract::Error::invalid().to_string()));
        }
    };
    http::launch(project, read_only, machine, no_open, major)
        .map_err(|error| crate::ForgeError::InvalidArgument(error.to_string()))
}
