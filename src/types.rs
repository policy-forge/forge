//! Shared domain types used across multiple modules.

/// Detected OSCAL model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalModelType {
    Catalog,
    ComponentDefinition,
    Profile,
}

impl OscalModelType {
    /// OSCAL-standard string key for this model type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::ComponentDefinition => "component-definition",
            Self::Profile => "profile",
        }
    }
}

impl std::fmt::Display for OscalModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oscal_model_type_display() {
        assert_eq!(OscalModelType::Catalog.to_string(), "catalog");
        assert_eq!(OscalModelType::ComponentDefinition.to_string(), "component-definition");
        assert_eq!(OscalModelType::Profile.to_string(), "profile");
    }

    #[test]
    fn oscal_model_type_as_str() {
        assert_eq!(OscalModelType::Catalog.as_str(), "catalog");
        assert_eq!(OscalModelType::ComponentDefinition.as_str(), "component-definition");
        assert_eq!(OscalModelType::Profile.as_str(), "profile");
    }
}
