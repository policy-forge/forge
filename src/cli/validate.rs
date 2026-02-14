use std::path::Path;

use crate::ForgeError;

/// Execute the validate subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the validation fails.
pub fn execute(_input: &Path) -> Result<(), ForgeError> {
    Err(ForgeError::Validation(
        "validate command not yet implemented — coming in a future release".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_returns_validation_error_stub() {
        let result = execute(Path::new("any.json"));
        let err = result.expect_err("validate stub should return Err");
        assert!(
            matches!(err, ForgeError::Validation(ref msg) if msg.contains("not yet implemented")),
            "Expected ForgeError::Validation containing 'not yet implemented', got: {err:?}"
        );
    }
}
