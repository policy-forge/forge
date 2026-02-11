use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ForgeError::Io(io_err);
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn parse_error_display() {
        let err = ForgeError::Parse("unexpected token".to_string());
        assert_eq!(err.to_string(), "Parse error: unexpected token");
    }

    #[test]
    fn validation_error_display() {
        let err = ForgeError::Validation("missing required field".to_string());
        assert_eq!(err.to_string(), "Validation error: missing required field");
    }

    #[test]
    fn config_error_display() {
        let err = ForgeError::Config("invalid setting".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid setting");
    }

    #[test]
    fn io_error_from_conversion() {
        fn produce_io_error() -> Result<(), ForgeError> {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
            Err(io_err)?
        }

        let result = produce_io_error();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ForgeError::Io(_)), "Expected ForgeError::Io, got: {err:?}");
        assert_eq!(err.to_string(), "I/O error: access denied");
    }
}
