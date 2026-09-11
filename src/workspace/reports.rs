//! Closed, inert export envelope. Report bytes bind inputs without copying labels or prose.
use super::contract::{self, Error, Result};
use super::index::Role;
use super::services::{Snapshot, resource_id};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const PREFIX: &str = "<!doctype html>\n<html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'\"><title>FORGE review report</title><main><h1>FORGE review report</h1><p>Local review facts. These counts do not establish implementation, effectiveness, approval, or compliance. Trace locations describe current captured sources; original source hashes are not asserted.</p><pre>";
const SUFFIX: &str = "</pre></main></html>\n";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: String,
    #[serde(rename = "report_kind")]
    pub(crate) kind: String,
    pub(crate) inputs: Vec<Pin>,
    summary: Value,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    resource_id: String,
    sha256: String,
}

fn pins(snapshot: &Snapshot) -> Result<Vec<Pin>> {
    let mut pins: Vec<_> = snapshot
        .items
        .iter()
        .filter(|item| {
            !matches!(item.registration.role, Role::TraceReport | Role::ApplicabilityReport)
        })
        .map(|item| Pin {
            resource_id: resource_id(&item.registration),
            sha256: item.captured.sha256.clone(),
        })
        .collect();
    if pins.len() > 100 {
        return Err(Error::invalid());
    }
    pins.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));
    Ok(pins)
}

pub(crate) fn render(snapshot: &Snapshot, kind: &str, summary: Value) -> Result<Vec<u8>> {
    let report = Report {
        schema_version: "forge.workspace-report/1".into(),
        kind: kind.into(),
        inputs: pins(snapshot)?,
        summary,
    };
    report.validate()?;
    report.bytes()
}
impl Report {
    fn validate(&self) -> Result<()> {
        contract::validate(
            "WorkspaceReportEnvelope",
            &serde_json::to_value(self).map_err(|_| Error::invalid())?,
        )?;
        if self.schema_version != "forge.workspace-report/1" || self.inputs.len() > 100 {
            return Err(Error::invalid());
        }
        let mut last = "";
        for pin in &self.inputs {
            contract::validate("ResourceId", &json!(pin.resource_id))?;
            contract::validate("Sha256Hex", &json!(pin.sha256))?;
            if pin.resource_id.as_str() <= last {
                return Err(Error::invalid());
            }
            last = &pin.resource_id;
        }
        let keys: &[&str] = match self.kind.as_str() {
            "trace" => &[
                "total_elements",
                "asserted_trace_elements",
                "current_source_locations",
                "unresolved_elements",
            ],
            "mapping-collection" => &["eligible_controls", "referenced_controls"],
            "applicability-gap" => {
                let values = self.summary.as_array().ok_or_else(Error::invalid)?;
                if values.len() != 6 {
                    return Err(Error::invalid());
                }
                let mut classes = std::collections::BTreeSet::new();
                for value in values {
                    if value.as_object().is_none_or(|value| value.len() != 2)
                        || value["count"].as_u64().is_none_or(|count| count > 10000)
                    {
                        return Err(Error::invalid());
                    }
                    contract::validate("GapClassification", &value["classification"])?;
                    if !classes.insert(value["classification"].as_str()) {
                        return Err(Error::invalid());
                    }
                }
                return Ok(());
            }
            _ => return Err(Error::invalid()),
        };
        let object = self.summary.as_object().ok_or_else(Error::invalid)?;
        if object.len() != keys.len()
            || keys.iter().any(|key| {
                object.get(*key).and_then(Value::as_u64).is_none_or(|count| count > 10000)
            })
        {
            return Err(Error::invalid());
        }
        if self.kind == "trace" {
            let n = |key: &str| self.summary[key].as_u64().unwrap_or_default();
            if n("asserted_trace_elements") > n("total_elements")
                || n("current_source_locations") > n("asserted_trace_elements")
                || n("current_source_locations") + n("unresolved_elements") != n("total_elements")
            {
                return Err(Error::invalid());
            }
        } else if self.summary["referenced_controls"].as_u64()
            > self.summary["eligible_controls"].as_u64()
        {
            return Err(Error::invalid());
        }
        Ok(())
    }
    fn bytes(&self) -> Result<Vec<u8>> {
        let json = contract::encode(self, 1024 * 1024, true)?;
        let text = String::from_utf8(json).map_err(|_| Error::invalid())?;
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\"', "&quot;")
            .replace('\'', "&#39;");
        Ok(format!("{PREFIX}{escaped}{SUFFIX}").into_bytes())
    }
    pub(crate) fn matches(&self, snapshot: &Snapshot) -> Result<bool> {
        Ok(self.inputs == pins(snapshot)?
            && self.kind == "trace"
            && self.summary == super::domain::trace_counts(snapshot)?)
    }
}
pub(crate) fn parse(bytes: &[u8]) -> Result<Report> {
    if bytes.len() > 1024 * 1024 {
        return Err(Error::invalid());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Error::invalid())?;
    let body = text
        .strip_prefix(PREFIX)
        .and_then(|text| text.strip_suffix(SUFFIX))
        .ok_or_else(Error::invalid)?;
    let json = body
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    let value = contract::parse(json.as_bytes(), 1024 * 1024, 1024)?;
    let report: Report = serde_json::from_value(value).map_err(|_| Error::invalid())?;
    report.validate()?;
    if report.bytes()? != bytes {
        return Err(Error::invalid());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_inert_reports_reject_injection_forward_versions_and_unbound_counts() {
        let dir = tempfile::tempdir().unwrap();
        let root = super::super::root::Root::open(dir.path()).unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        let summary = json!({"total_elements":0,"asserted_trace_elements":0,"current_source_locations":0,"unresolved_elements":0});
        let bytes = render(&snapshot, "trace", summary.clone()).unwrap();
        assert!(parse(&bytes).unwrap().matches(&snapshot).unwrap());
        for replacement in ["forge.workspace-report/2", "<script>alert(1)</script>"] {
            let changed = String::from_utf8(bytes.clone())
                .unwrap()
                .replace("forge.workspace-report/1", replacement);
            assert!(parse(changed.as_bytes()).is_err());
        }
        let mut extra = summary.clone();
        extra["reviewer_name"] = json!("private");
        assert!(render(&snapshot, "trace", extra).is_err());
        let mut wrong = summary;
        wrong["current_source_locations"] = json!(1);
        assert!(render(&snapshot, "trace", wrong).is_err());
        let changed = String::from_utf8(bytes).unwrap().replace("</pre>", "</pre><img src=x>");
        assert!(parse(changed.as_bytes()).is_err());
    }
}
