//! Local workspace services and adapters. Project files remain authoritative.

pub(crate) mod actions;
pub(crate) mod assets;
pub(crate) mod contract;
pub(crate) mod domain;
pub(crate) mod effects;
pub(crate) mod http;
pub(crate) mod index;
pub(crate) mod provenance;
pub(crate) mod reports;
pub(crate) mod root;
pub(crate) mod services;
pub(crate) mod session;

pub(crate) fn launch(
    project: &std::path::Path,
    read_only: bool,
    machine: bool,
    no_open: bool,
) -> Result<(), crate::ForgeError> {
    http::launch(project, read_only, machine, no_open)
        .map_err(|error| crate::ForgeError::InvalidArgument(error.to_string()))
}
