//! oscal-cli conversion chain: JSON → XML → YAML → JSON.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::ForgeError;
use crate::oscal_cli::{ConvertArgs, OscalCliInvoke, OscalFormat};

/// Execute the full oscal-cli conversion chain: JSON → XML → YAML → JSON.
///
/// Creates uniquely named intermediate files in `temp_dir`. Each conversion
/// step has an independent `timeout`.
///
/// # Errors
///
/// Returns [`ForgeError::RoundTripStep`] with step and path context if a
/// conversion step fails or times out.
pub fn run_round_trip_chain(
    input_json_path: &Path,
    invoker: &dyn OscalCliInvoke,
    temp_dir: &Path,
    timeout: Duration,
) -> Result<PathBuf, ForgeError> {
    let run_id = ::uuid::Uuid::new_v4();
    let xml_path = temp_dir.join(format!("artifact-{run_id}.xml"));
    let yaml_path = temp_dir.join(format!("artifact-{run_id}.yaml"));
    let rt_json_path = temp_dir.join(format!("artifact-{run_id}-rt.json"));
    let targets = [&xml_path, &yaml_path, &rt_json_path];

    for target in targets {
        let _ = std::fs::remove_file(target);
    }

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

        match invoker.convert(&args) {
            Ok(result) => {
                for warning in result.warnings {
                    tracing::warn!(step = step_num + 1, "oscal-cli convert warning: {warning}");
                }
            }
            Err(source) => {
                for target in targets {
                    let _ = std::fs::remove_file(target);
                }
                return Err(ForgeError::RoundTripStep {
                    step: step_num + 1,
                    from: input.to_path_buf(),
                    to: output.to_path_buf(),
                    source: Box::new(source),
                });
            }
        }
    }

    Ok(rt_json_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oscal_cli::{ConvertArgs, ConvertResult, ResolveArgs, ResolveResult};
    use std::sync::Mutex;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedCall {
        format: String,
        input: PathBuf,
        output: PathBuf,
        timeout: Duration,
    }

    /// Mock invoker that records every conversion request and returns canned results.
    struct MockInvoker {
        calls: Mutex<Vec<RecordedCall>>,
        fail_on_call: Option<usize>,
    }

    impl MockInvoker {
        fn new() -> Self {
            Self { calls: Mutex::new(Vec::new()), fail_on_call: None }
        }

        fn failing_on_call(call_number: usize) -> Self {
            Self { calls: Mutex::new(Vec::new()), fail_on_call: Some(call_number) }
        }

        fn calls(&self) -> Vec<RecordedCall> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl OscalCliInvoke for MockInvoker {
        fn resolve_profile(&self, _args: &ResolveArgs) -> Result<ResolveResult, ForgeError> {
            unimplemented!("MockInvoker does not support resolve_profile")
        }

        fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError> {
            let mut calls = self.calls.lock().unwrap();
            let call = calls.len() + 1;
            calls.push(RecordedCall {
                format: args.output_format.to_cli_flag().to_string(),
                input: args.input_path.clone(),
                output: args.output_path.clone(),
                timeout: args.timeout,
            });
            drop(calls);

            if self.fail_on_call == Some(call) {
                return Err(ForgeError::OscalCliTimeout { timeout: args.timeout });
            }

            // Create an empty output file to simulate conversion.
            std::fs::write(&args.output_path, b"{}").map_err(ForgeError::Io)?;

            Ok(ConvertResult { output_path: args.output_path.clone(), warnings: vec![] })
        }
    }

    fn assert_expected_chain(
        calls: &[RecordedCall],
        input: &Path,
        final_output: &Path,
        timeout: Duration,
    ) {
        let prefix = final_output
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|stem| stem.strip_suffix("-rt"))
            .expect("round-trip output has the expected file name");
        let temp_dir = final_output.parent().expect("round-trip output has a parent directory");
        let xml_path = temp_dir.join(format!("{prefix}.xml"));
        let yaml_path = temp_dir.join(format!("{prefix}.yaml"));

        assert_eq!(
            calls,
            &[
                RecordedCall {
                    format: "xml".to_string(),
                    input: input.to_path_buf(),
                    output: xml_path.clone(),
                    timeout,
                },
                RecordedCall {
                    format: "yaml".to_string(),
                    input: xml_path,
                    output: yaml_path.clone(),
                    timeout,
                },
                RecordedCall {
                    format: "json".to_string(),
                    input: yaml_path,
                    output: final_output.to_path_buf(),
                    timeout,
                },
            ]
        );
    }

    #[test]
    fn happy_path_uses_the_expected_conversion_chain() {
        let invoker = MockInvoker::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.json");
        std::fs::write(&input, b"{}").unwrap();
        let timeout = Duration::from_secs(30);

        let output_path = run_round_trip_chain(&input, &invoker, temp_dir.path(), timeout).unwrap();

        assert!(output_path.exists(), "Final output file should exist");
        assert_expected_chain(&invoker.calls(), &input, &output_path, timeout);
    }

    #[test]
    fn failed_step_is_contextualized_and_removes_intermediates() {
        let invoker = MockInvoker::failing_on_call(2);
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.json");
        std::fs::write(&input, b"{}").unwrap();

        let error =
            run_round_trip_chain(&input, &invoker, temp_dir.path(), Duration::from_secs(30))
                .unwrap_err();
        let calls = invoker.calls();

        match error {
            ForgeError::RoundTripStep { step, from, to, .. } => {
                assert_eq!(step, 2);
                assert_eq!(from, calls[0].output);
                assert_eq!(to, calls[1].output);
            }
            other => panic!("expected round-trip step error, got {other:?}"),
        }
        assert!(calls.iter().all(|call| !call.output.exists()));
    }

    #[test]
    fn concurrent_runs_use_distinct_intermediate_paths() {
        let invoker = MockInvoker::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.json");
        std::fs::write(&input, b"{}").unwrap();

        let (first, second) = std::thread::scope(|scope| {
            let first = scope.spawn(|| {
                run_round_trip_chain(&input, &invoker, temp_dir.path(), Duration::from_secs(30))
            });
            let second = scope.spawn(|| {
                run_round_trip_chain(&input, &invoker, temp_dir.path(), Duration::from_secs(30))
            });
            (first.join().unwrap().unwrap(), second.join().unwrap().unwrap())
        });

        assert_ne!(first, second);
        assert!(first.exists());
        assert!(second.exists());
    }
}
