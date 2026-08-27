//! Shared domain types used across multiple modules.

use clap::ValueEnum;

/// Detected OSCAL model type.
///
/// Distinguishes between the OSCAL document types that FORGE can
/// detect and process: [Catalog][crate::oscal::catalog::OscalCatalog],
/// [`ComponentDefinition`][crate::oscal::component_definition::ComponentDefinition], and
/// [Profile][crate::oscal::profile::OscalProfile], plus Control Mapping collections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalModelType {
    /// An OSCAL Catalog containing groups and controls.
    Catalog,
    /// An OSCAL Component Definition describing implemented requirements.
    ComponentDefinition,
    /// An OSCAL Profile that selects/overrides controls from a Catalog.
    Profile,
    /// An OSCAL Control Mapping collection.
    Mapping,
}

impl OscalModelType {
    /// OSCAL-standard string key for this model type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::ComponentDefinition => "component-definition",
            Self::Profile => "profile",
            Self::Mapping => "mapping-collection",
        }
    }
}

impl std::fmt::Display for OscalModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.as_str())
    }
}

/// Conversion strategy: which OSCAL model to produce.
///
/// Determines the target OSCAL artifact type when converting a policy document.
/// `Catalog` produces an OSCAL Catalog with groups and controls;
/// `Component` produces an OSCAL Component Definition with implemented requirements.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    /// Produce an OSCAL Catalog (groups → controls → statements).
    Catalog,
    /// Produce an OSCAL Component Definition (implemented requirements).
    ///
    /// The CLI value is deliberately the concise `component`; `--to` uses the
    /// full `component-definition` artifact name.
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

/// Output type: which OSCAL artifact to produce from a policy document.
///
/// Used by `forge convert --to <type>` to select the target artifact.
/// When `--to` is omitted, the `Strategy` enum determines the output type
/// (`catalog` or `component-definition`).
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputType {
    /// Produce an OSCAL Catalog (groups → controls → statements).
    Catalog,
    /// Produce an OSCAL Component Definition (implemented requirements).
    #[value(name = "component-definition")]
    ComponentDefinition,
    /// Produce an OSCAL System Security Plan (SSP) skeleton.
    Ssp,
}

impl OutputType {
    /// Human-readable label for this output type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::ComponentDefinition => "component-definition",
            Self::Ssp => "ssp",
        }
    }
}

impl std::fmt::Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.as_str())
    }
}

/// Output serialization format.
///
/// Represents the target format for serialized OSCAL output.
/// Supported formats: JSON, XML, and YAML.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON output format (default).
    Json,
    /// XML output format.
    Xml,
    /// YAML output format.
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
        assert_eq!(OscalModelType::Mapping.to_string(), "mapping-collection");
    }

    #[test]
    fn oscal_model_type_as_str() {
        assert_eq!(OscalModelType::Catalog.as_str(), "catalog");
        assert_eq!(OscalModelType::ComponentDefinition.as_str(), "component-definition");
        assert_eq!(OscalModelType::Profile.as_str(), "profile");
        assert_eq!(OscalModelType::Mapping.as_str(), "mapping-collection");
    }

    #[test]
    fn strategy_display_catalog() {
        assert_eq!(Strategy::Catalog.to_string(), "catalog");
    }

    #[test]
    fn strategy_display_component() {
        assert_eq!(Strategy::Component.to_string(), "component");
    }

    #[test]
    fn output_type_display() {
        assert_eq!(OutputType::Catalog.to_string(), "catalog");
        assert_eq!(OutputType::ComponentDefinition.to_string(), "component-definition");
        assert_eq!(OutputType::Ssp.to_string(), "ssp");
    }

    #[test]
    fn output_type_as_str() {
        assert_eq!(OutputType::Catalog.as_str(), "catalog");
        assert_eq!(OutputType::ComponentDefinition.as_str(), "component-definition");
        assert_eq!(OutputType::Ssp.as_str(), "ssp");
    }
}
