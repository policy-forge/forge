use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};

use super::summary::{BatchSummary, FileResult};

/// Validate that all input files exist and are regular files.
///
/// Returns `Ok(())` if all valid, or `Err` listing all invalid paths with reasons.
///
/// # Errors
///
/// Returns `ForgeError::BatchConversion` if any input path is missing or not a file.
pub fn validate_inputs(input_paths: &[PathBuf]) -> Result<(), ForgeError> {
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

    let results = if jobs == 0 {
        // Use rayon's global thread pool (already initialized, no allocation cost)
        path_pairs.par_iter().map(convert).collect()
    } else {
        match rayon::ThreadPoolBuilder::new().num_threads(jobs).build() {
            Ok(pool) => pool.install(|| path_pairs.par_iter().map(convert).collect()),
            Err(e) => {
                tracing::error!("Failed to create thread pool: {e}. Falling back to sequential.");
                path_pairs.iter().map(convert).collect()
            }
        }
    };

    BatchSummary::from_results(results, batch_start.elapsed())
}

/// Convert a single file with panic isolation.
fn convert_single_file(
    input: &Path,
    output: &Path,
    strategy: Strategy,
    format: OutputFormat,
    max_size_bytes: u64,
    source_profile: Option<&str>,
) -> FileResult {
    let start = Instant::now();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_pipeline(input, output, strategy, format, max_size_bytes, source_profile)
    }));

    let duration = start.elapsed();

    match result {
        Ok(Ok(())) => FileResult::success(input.to_path_buf(), output.to_path_buf(), duration),
        Ok(Err(e)) => FileResult::failure(input.to_path_buf(), e.to_string(), duration),
        Err(_) => FileResult::failure(
            input.to_path_buf(),
            "Internal error (panic during conversion)".to_string(),
            duration,
        ),
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
    match strategy {
        Strategy::Catalog => {
            crate::pipeline::run_catalog_pipeline(input, Some(output), max_size_bytes, &format)?;
        }
        Strategy::Component => {
            crate::pipeline::run_component_pipeline(
                input,
                Some(output),
                max_size_bytes,
                source_profile,
                &format,
            )?;
        }
    }
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
    fn validate_inputs_empty_is_ok() {
        // Empty input is valid at this layer — caller owns the emptiness check
        let result = validate_inputs(&[]);
        assert!(result.is_ok());
    }
}
