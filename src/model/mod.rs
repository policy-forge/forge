/// A single atomic policy requirement extracted from a source document.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRequirement {
    /// Raw requirement text extracted from the policy document.
    pub text: String,
    /// 1-based line number in the source document.
    pub source_line: usize,
    /// 0-based nesting depth within the document hierarchy.
    pub nesting_depth: u8,
    /// Deterministic UUID v5 identifier, populated by the uuid module (WI-7).
    /// `None` before UUID generation, `Some(uuid_string)` after.
    pub stable_id: Option<String>,
}

/// A section within a policy document, containing requirements and nested subsections.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySection {
    /// Section heading text.
    pub title: String,
    /// Requirements within this section.
    pub requirements: Vec<PolicyRequirement>,
    /// Nested subsections.
    pub children: Vec<PolicySection>,
}

/// A parsed policy document with a hierarchical structure of sections and requirements.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDocument {
    /// Top-level sections with nested hierarchy.
    pub sections: Vec<PolicySection>,
}
