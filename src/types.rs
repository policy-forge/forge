//! Shared domain types used across multiple modules.

use clap::ValueEnum;

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
        f.pad(self.as_str())
    }
}

/// Conversion strategy: which OSCAL model to produce.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Catalog,
    Component,
}

impl Strategy {
    /// Human-readable label for this strategy.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Component => "component",
        }
    }
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.as_str())
    }
}

/// Output serialization format.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Xml,
    Yaml,
}

impl OutputFormat {
    /// The canonical file extension for this output format.
    #[must_use]
    pub fn as_extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Yaml => "yaml",
        }
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

    #[test]
    fn strategy_display_catalog() {
        assert_eq!(Strategy::Catalog.to_string(), "catalog");
    }

    #[test]
    fn strategy_display_component() {
        assert_eq!(Strategy::Component.to_string(), "component");
    }
}
