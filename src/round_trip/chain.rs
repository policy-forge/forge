//! oscal-cli conversion chain: JSON → XML → YAML → JSON.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::ForgeError;
use crate::oscal_cli::{ConvertArgs, OscalCliInvoke, OscalFormat};

/// Execute the full oscal-cli conversion chain: JSON → XML → YAML → JSON.
///
/// Creates intermediate files in `temp_dir` (artifact.xml, artifact.yaml, artifact-rt.json).
/// Each conversion step has an independent `timeout`.
///
/// # Errors
///
/// Returns `ForgeError` if any conversion step fails or times out.
pub fn run_round_trip_chain(
    input_json_path: &Path,
    invoker: &dyn OscalCliInvoke,
    temp_dir: &Path,
    timeout: Duration,
) -> Result<PathBuf, ForgeError> {
    let xml_path = temp_dir.join("artifact.xml");
    let yaml_path = temp_dir.join("artifact.yaml");
    let rt_json_path = temp_dir.join("artifact-rt.json");

    let steps: &[(OscalFormat, &Path, &Path)] = &[
        (OscalFormat::Xml, input_json_path, &xml_path),
        (OscalFormat::Yaml, &xml_path, &yaml_path),
        (OscalFormat::Json, &yaml_path, &rt_json_path),
    ];

    for (step_num, (format, input, output)) in steps.iter().enumerate() {
        tracing::debug!(
            step = step_num + 1,
            from = %input.display(),
            to = %output.display(),
            format = format.to_cli_flag(),
            "Running oscal-cli convert step"
        );

        let args = ConvertArgs {
            input_path: input.to_path_buf(),
            output_path: output.to_path_buf(),
            output_format: *format,
            timeout,
        };

        invoker.convert(&args)?;
    }

    Ok(rt_json_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oscal_cli::{ConvertArgs, ConvertResult, ResolveArgs, ResolveResult};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock invoker that returns canned results for testing.
    struct MockInvoker {
        call_count: AtomicUsize,
        fail_on_call: Option<usize>,
    }

    impl MockInvoker {
        fn new() -> Self {
            Self { call_count: AtomicUsize::new(0), fail_on_call: None }
        }

        fn failing_on_call(call_number: usize) -> Self {
            Self { call_count: AtomicUsize::new(0), fail_on_call: Some(call_number) }
        }
    }

    impl OscalCliInvoke for MockInvoker {
        fn resolve_profile(&self, _args: &ResolveArgs) -> Result<ResolveResult, ForgeError> {
            unimplemented!("MockInvoker does not support resolve_profile")
        }

        fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError> {
            let call = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;

            if self.fail_on_call == Some(call) {
                return Err(ForgeError::OscalCliTimeout { timeout: args.timeout });
            }

            // Create an empty output file to simulate conversion
            std::fs::write(&args.output_path, b"{}").map_err(ForgeError::Io)?;

            Ok(ConvertResult { output_path: args.output_path.clone(), warnings: vec![] })
        }
    }

    // T010(a): Happy path — chain calls convert() 3 times and returns final output path
    #[test]
    fn happy_path_calls_convert_three_times() {
        let invoker = MockInvoker::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.json");
        std::fs::write(&input, b"{}").unwrap();

        let result =
            run_round_trip_chain(&input, &invoker, temp_dir.path(), Duration::from_secs(30));
        assert!(result.is_ok(), "Happy path should succeed: {result:?}");

        let output_path = result.unwrap();
        assert!(output_path.exists(), "Final output file should exist");
        assert_eq!(
            invoker.call_count.load(Ordering::SeqCst),
            3,
            "Should call convert exactly 3 times"
        );
    }

    // T010(b): Timeout on second call → returns OscalCliTimeout error
    #[test]
    fn timeout_on_second_step_returns_error() {
        let invoker = MockInvoker::failing_on_call(2);
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.json");
        std::fs::write(&input, b"{}").unwrap();

        let result =
            run_round_trip_chain(&input, &invoker, temp_dir.path(), Duration::from_secs(30));
        assert!(result.is_err(), "Should return error on timeout");
        assert!(
            matches!(result.unwrap_err(), ForgeError::OscalCliTimeout { .. }),
            "Error should be OscalCliTimeout"
        );
    }

    // T020: Verify all three intermediate files are created with correct extensions
    #[test]
    fn intermediate_files_have_correct_extensions() {
        let invoker = MockInvoker::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.json");
        std::fs::write(&input, b"{}").unwrap();

        let result =
            run_round_trip_chain(&input, &invoker, temp_dir.path(), Duration::from_secs(30));
        assert!(result.is_ok());

        let xml_path = temp_dir.path().join("artifact.xml");
        let yaml_path = temp_dir.path().join("artifact.yaml");
        let rt_json_path = temp_dir.path().join("artifact-rt.json");

        assert!(xml_path.exists(), "Intermediate XML file should exist");
        assert!(yaml_path.exists(), "Intermediate YAML file should exist");
        assert!(rt_json_path.exists(), "Final round-tripped JSON file should exist");

        // Verify the returned path is the final JSON
        assert_eq!(result.unwrap(), rt_json_path);
    }
}
