//! Deterministic offline views of validated, value-redacted authoring evidence.
//!
//! All data is displayed as text, including apparent URLs and Markdown. No
//! attribute or active HTML is ever formed from an input. JSON-byte adapters
//! are crate-private and may consume only freshly rendered validated reports;
//! they are not import or provenance-validation APIs.

use super::{error, model::AuthoringPlan, output::MAX_OUTPUT_BYTES};
use crate::ForgeError;

/// Render a validated authoring plan without organization answer values.
pub(crate) fn render_plan_bounded(
    plan: &AuthoringPlan,
    limit: usize,
) -> Result<Vec<u8>, ForgeError> {
    render_document("Authoring plan", &bounded_json(plan, limit)?, limit)
}

/// Render the validated impact report, including unsupported comparisons.
pub(crate) fn render_impact_bounded(
    report: &super::impact::ImpactReport,
    limit: usize,
) -> Result<Vec<u8>, ForgeError> {
    render_document("Authoring impact", &bounded_json(report, limit)?, limit)
}

/// Consume only the redacted provenance bytes freshly produced by the renderer.
pub(crate) fn render_provenance_bounded(bytes: &[u8], limit: usize) -> Result<Vec<u8>, ForgeError> {
    require_schema(
        bytes,
        &["forge.authoring-provenance/1", "forge.authoring-provenance/2"],
        limit,
    )?;
    render_document("Authoring provenance", bytes, limit)
}

/// Consume only the redacted `/2` plan freshly produced by component integration.
pub(crate) fn render_component_plan_bounded(
    bytes: &[u8],
    limit: usize,
) -> Result<Vec<u8>, ForgeError> {
    require_schema(bytes, &["forge.authoring-plan/2"], limit)?;
    render_document("Authoring plan", bytes, limit)
}

fn require_schema(bytes: &[u8], allowed: &[&str], limit: usize) -> Result<(), ForgeError> {
    if bytes.len() > limit.min(MAX_OUTPUT_BYTES) {
        return Err(error("HTML report input exceeds its byte limit"));
    }
    let value = crate::json_strict::parse_value(
        bytes,
        "HTML report",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: 16 * 1024 },
    )
    .map_err(|cause| error(cause.to_string()))?;
    if !value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|version| allowed.contains(&version))
    {
        return Err(error("HTML view received an unsupported report contract"));
    }
    reject_raw_values(&value)
}

fn reject_raw_values(value: &serde_json::Value) -> Result<(), ForgeError> {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                if matches!(
                    key.as_str(),
                    "value"
                        | "values"
                        | "markdown"
                        | "parameter_values"
                        | "answer_values"
                        | "raw_answer"
                        | "raw_parameter"
                ) {
                    return Err(error("HTML views require value-redacted report data"));
                }
                reject_raw_values(value)?;
            }
            Ok(())
        }
        serde_json::Value::Array(values) => values.iter().try_for_each(reject_raw_values),
        _ => Ok(()),
    }
}

fn render_document(title: &str, bytes: &[u8], limit: usize) -> Result<Vec<u8>, ForgeError> {
    if bytes.len() > limit.min(MAX_OUTPUT_BYTES) {
        return Err(error("HTML report input exceeds its byte limit"));
    }
    let json = std::str::from_utf8(bytes).map_err(|_| error("HTML report must be UTF-8"))?;
    let mut output = BoundedHtml { bytes: Vec::new(), limit: limit.min(MAX_OUTPUT_BYTES) };
    output.literal("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'\">\n<title>")?;
    output.escaped(title)?;
    output.literal("</title>\n</head>\n<body>\n<main>\n<h1>")?;
    output.escaped(title)?;
    output.literal("</h1>\n<p>Drafting evidence only. Unresolved work and authoring states remain visible. Human review remains pending.</p>\n<p>Reviewer and component status metadata are supplied assertions. Answer and parameter values are omitted.</p>\n<pre>")?;
    output.escaped(json)?;
    output.literal("</pre>\n</main>\n</body>\n</html>\n")?;
    Ok(output.bytes)
}

fn bounded_json(value: &impl serde::Serialize, limit: usize) -> Result<Vec<u8>, ForgeError> {
    struct Writer {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl std::io::Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
                return Err(std::io::Error::other(
                    "HTML input exceeds remaining generation budget",
                ));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Writer { bytes: Vec::new(), limit: limit.min(MAX_OUTPUT_BYTES) };
    serde_json::to_writer_pretty(&mut writer, value)
        .map_err(|cause| error(format!("cannot encode HTML input: {cause}")))?;
    std::io::Write::write_all(&mut writer, b"\n")
        .map_err(|cause| error(format!("cannot finish HTML input: {cause}")))?;
    Ok(writer.bytes)
}

struct BoundedHtml {
    bytes: Vec<u8>,
    limit: usize,
}
impl BoundedHtml {
    fn literal(&mut self, value: &str) -> Result<(), ForgeError> {
        if value.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(error("HTML report exceeds the output byte limit"));
        }
        self.bytes.extend_from_slice(value.as_bytes());
        Ok(())
    }
    fn escaped(&mut self, value: &str) -> Result<(), ForgeError> {
        for character in value.chars() {
            match character {
                '&' => self.literal("&amp;")?,
                '<' => self.literal("&lt;")?,
                '>' => self.literal("&gt;")?,
                '"' => self.literal("&quot;")?,
                '\'' => self.literal("&#39;")?,
                other => {
                    let mut bytes = [0u8; 4];
                    self.literal(other.encode_utf8(&mut bytes))?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_provenance(bytes: &[u8]) -> Result<Vec<u8>, ForgeError> {
        render_provenance_bounded(bytes, MAX_OUTPUT_BYTES)
    }
    fn render_component_plan(bytes: &[u8]) -> Result<Vec<u8>, ForgeError> {
        render_component_plan_bounded(bytes, MAX_OUTPUT_BYTES)
    }

    #[test]
    fn every_label_is_text_and_unicode_and_offline_bytes_are_deterministic() {
        let input = serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version":"forge.authoring-provenance/2",
            "title":"</pre><script src='https://example.test'>bad</script>",
            "rationale":"<img src=x onerror=alert(1)>",
            "filename":"javascript:alert(1)",
            "component_metadata":"<form action='https://example.test'>秘密 & café</form>",
            "state":"blocked-context"
        }))
        .unwrap();
        let first = render_provenance(&input).unwrap();
        assert_eq!(first, render_provenance(&input).unwrap());
        let text = String::from_utf8(first).unwrap();
        for forbidden in ["<script", "<img", "<form", "<a ", "<link", "<style"] {
            assert!(!text.contains(forbidden), "{forbidden}");
        }
        assert!(text.contains("&lt;/pre&gt;&lt;script"));
        assert!(text.contains("秘密 &amp; café"));
        assert!(text.contains("blocked-context"));
        assert!(text.contains("default-src 'none'"));
    }

    #[test]
    fn raw_values_and_unsupported_or_duplicate_contracts_are_rejected() {
        for key in ["value", "values", "markdown", "answer_values", "parameter_values"] {
            let input = serde_json::to_vec(&serde_json::json!({"schema_version":"forge.authoring-provenance/2", "nested": {key: "sensitive"}})).unwrap();
            assert!(render_provenance(&input).is_err());
        }
        assert!(
            render_provenance(br#"{"schema_version":"forge.authoring-provenance/3"}"#).is_err()
        );
        assert!(render_provenance(br#"{"schema_version":"forge.authoring-provenance/1","schema_version":"forge.authoring-provenance/1"}"#).is_err());
        assert!(render_component_plan(br#"{"schema_version":"forge.authoring-plan/1"}"#).is_err());
    }

    #[test]
    fn escaping_stops_before_allocating_beyond_the_limit() {
        let mut output = BoundedHtml { bytes: Vec::new(), limit: 9 };
        assert!(output.escaped("&&").is_err());
        assert_eq!(output.bytes, b"&amp;");
        let mut output = BoundedHtml { bytes: Vec::new(), limit: 4 };
        output.escaped("🦀").unwrap();
        assert!(output.escaped("x").is_err());
        let input = br#"{"schema_version":"forge.authoring-provenance/1"}"#;
        let rendered = render_provenance(input).unwrap();
        assert!(render_provenance_bounded(input, rendered.len() - 1).is_err());
        assert_eq!(render_provenance_bounded(input, rendered.len()).unwrap(), rendered);
        assert!(bounded_json(&serde_json::json!({"label":"long"}), 4).is_err());
    }
}
