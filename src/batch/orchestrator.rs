use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};

use super::summary::{BatchSummary, FileResult};

/// Validate input paths before conversion.
///
/// The CLI calls this before dispatching batch work. `run_batch_conversion` accepts
/// prevalidated pairs so programmatic callers can choose their own validation boundary.
///
/// Returns `Ok(())` if all paths are valid, or `Err` listing all invalid paths
/// with reasons.
///
/// # Errors
///
/// Returns `ForgeError::BatchConversion` if:
/// - `input_paths` is empty ("No input files provided")
/// - Any input path does not exist or is not a regular file
pub fn validate_inputs(input_paths: &[PathBuf]) -> Result<(), ForgeError> {
    if input_paths.is_empty() {
        return Err(ForgeError::BatchConversion("No input files provided".to_string()));
    }

    let mut invalid: Vec<String> = Vec::new();

    for path in input_paths {
        if !path.exists() {
            invalid.push(format!("'{}': not found", path.display()));
        } else if !path.is_file() {
            invalid.push(format!("'{}': not a regular file", path.display()));
        }
    }

    if invalid.is_empty() {
        Ok(())
    } else {
        Err(ForgeError::BatchConversion(format!(
            "Invalid input files:\n  {}",
            invalid.join("\n  ")
        )))
    }
}

/// Run batch conversion on multiple input files.
///
/// Uses rayon for parallel processing. `jobs` = 0 means auto (global thread pool),
/// `jobs` >= 1 means a custom pool with that many threads.
/// Each file is converted independently; a failure in one does not affect others.
#[must_use]
pub fn run_batch_conversion(
    path_pairs: &[(PathBuf, PathBuf)],
    strategy: Strategy,
    format: OutputFormat,
    max_size_bytes: u64,
    source_profile: Option<&str>,
    jobs: usize,
) -> BatchSummary {
    let batch_start = Instant::now();

    tracing::info!(
        file_count = path_pairs.len(),
        parallelism = if jobs == 0 { "auto".to_string() } else { jobs.to_string() },
        "Entering batch conversion mode"
    );

    // Warn on large batches (FR-016)
    if path_pairs.len() > 100 {
        tracing::warn!("Large batch: {} files. Processing may take a while.", path_pairs.len());
    }

    let convert = |(input, output): &(PathBuf, PathBuf)| {
        convert_single_file(input, output, strategy, format, max_size_bytes, source_profile)
    };

    let (results, degraded_to_sequential) = if jobs == 0 {
        // Use rayon's global thread pool (already initialized, no allocation cost)
        (path_pairs.par_iter().map(convert).collect(), false)
    } else {
        match rayon::ThreadPoolBuilder::new().num_threads(jobs).build() {
            Ok(pool) => (pool.install(|| path_pairs.par_iter().map(convert).collect()), false),
            Err(error) => {
                tracing::error!(
                    "Failed to create thread pool: {error}. Falling back to sequential."
                );
                (path_pairs.iter().map(convert).collect(), true)
            }
        }
    };

    BatchSummary::from_results_with_execution_mode(
        results,
        batch_start.elapsed(),
        degraded_to_sequential,
    )
}

/// Convert a single file with panic isolation.
///
/// `run_pipeline` must remain free of shared mutable state: catching a panic is
/// sound only when a failed conversion cannot leave state shared with sibling
/// batch tasks inconsistent. With `panic = "abort"`, this isolation cannot return.
fn convert_single_file(
    input: &Path,
    output: &Path,
    strategy: Strategy,
    format: OutputFormat,
    max_size_bytes: u64,
    source_profile: Option<&str>,
) -> FileResult {
    convert_single_file_with(input, output, || {
        run_pipeline(input, output, strategy, format, max_size_bytes, source_profile)
    })
}

fn convert_single_file_with(
    input: &Path,
    output: &Path,
    convert: impl FnOnce() -> Result<(), ForgeError>,
) -> FileResult {
    let start = Instant::now();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(convert));
    let duration = start.elapsed();

    match result {
        Ok(Ok(())) => FileResult::success(input.to_path_buf(), output.to_path_buf(), duration),
        Ok(Err(error)) => {
            FileResult::failure(input.to_path_buf(), error.with_max_size_guidance(), duration)
        }
        Err(payload) => {
            let panic_detail = if let Some(message) = payload.downcast_ref::<&str>() {
                *message
            } else if let Some(message) = payload.downcast_ref::<String>() {
                message.as_str()
            } else {
                "non-string panic payload"
            };
            tracing::error!(input = %input.display(), panic_detail, "batch conversion panicked");
            FileResult::failure(
                input.to_path_buf(),
                ForgeError::BatchConversion("Internal error (panic during conversion)".to_string()),
                duration,
            )
        }
    }
}

/// Run the appropriate pipeline for a single file.
fn run_pipeline(
    input: &Path,
    output: &Path,
    strategy: Strategy,
    format: OutputFormat,
    max_size_bytes: u64,
    source_profile: Option<&str>,
) -> Result<(), ForgeError> {
    let result = match strategy {
        Strategy::Catalog => {
            crate::pipeline::run_catalog_pipeline(input, max_size_bytes, &format, None)?
        }
        Strategy::Component => crate::pipeline::run_component_pipeline(
            input,
            max_size_bytes,
            source_profile,
            &format,
            None,
        )?,
    };
    crate::cli::output::write_output(&result.content, Some(output))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // T014: validate_inputs tests

    #[test]
    fn validate_inputs_all_valid() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.md");
        let f2 = dir.path().join("b.md");
        std::fs::write(&f1, "# A").unwrap();
        std::fs::write(&f2, "# B").unwrap();

        let result = validate_inputs(&[f1, f2]);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_inputs_one_missing() {
        let dir = TempDir::new().unwrap();
        let f1 = dir.path().join("a.md");
        std::fs::write(&f1, "# A").unwrap();
        let missing = dir.path().join("nonexistent.md");

        let result = validate_inputs(&[f1, missing]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nonexistent.md"));
    }

    #[test]
    fn validate_inputs_empty_returns_error() {
        let result = validate_inputs(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No input files"));
    }

    #[test]
    fn panic_isolated_conversion_returns_generic_file_failure() {
        let result = convert_single_file_with(
            Path::new("input.md"),
            Path::new("output.json"),
            || -> Result<(), ForgeError> { panic!("injected panic") },
        );

        assert!(!result.is_success());
        assert!(matches!(
            result.outcome,
            crate::batch::summary::FileOutcome::Failure { ref error }
                if error.to_string().contains("Internal error (panic during conversion)")
        ));
    }

    #[test]
    fn single_worker_pool_converts_batch_without_fallback() {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("policy.md");
        let output = dir.path().join("policy.json");
        std::fs::write(
            &input,
            "# Policy\n\n## Access Control\n\n- The organization shall protect systems.\n",
        )
        .unwrap();

        let summary = run_batch_conversion(
            &[(input, output)],
            Strategy::Catalog,
            OutputFormat::Json,
            1024 * 1024,
            None,
            1,
        );

        assert_eq!(summary.total_files(), 1);
        assert_eq!(summary.succeeded(), 1);
        assert!(!summary.degraded_to_sequential());
    }
}
