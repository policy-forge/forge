// Contract: Exit code mapping (WI-23)
// Maps ForgeError variants to CLI exit codes.
// This function must be exhaustive over all ForgeError variants.

use crate::error::ForgeError;

/// Map a `ForgeError` to a CLI exit code.
///
/// Exit code categories:
/// - 0: Success (not handled here — only error cases)
/// - 1: Input/IO errors (file not found, permission denied, empty, binary, encoding, size, I/O)
/// - 2: Parse/Structure errors (no structure, parse failure, build errors)
/// - 3: Validation/Config errors (schema violations, config issues)
pub fn exit_code(err: &ForgeError) -> u8 {
    match err {
        // Exit 1: Input/IO errors
        ForgeError::FileNotFound { .. }
        | ForgeError::PermissionDenied { .. }
        | ForgeError::EmptyInput { .. }
        | ForgeError::BinaryFile { .. }
        | ForgeError::UnsupportedFormat { .. }
        | ForgeError::FileTooLarge { .. }
        | ForgeError::InvalidEncoding { .. }
        | ForgeError::NotAFile { .. }
        | ForgeError::Io(_)
        | ForgeError::Serialization(_) => 1,

        // Exit 2: Parse/Structure errors
        ForgeError::NoStructureDetected { .. }
        | ForgeError::Parse(_)
        | ForgeError::CatalogBuild(_)
        | ForgeError::BackMatter(_)
        | ForgeError::ComponentDefinitionBuild(_) => 2,

        // Exit 3: Validation/Config errors
        ForgeError::Validation(_) | ForgeError::Config(_) => 3,
    }
}
