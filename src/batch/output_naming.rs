use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::cli::OutputFormat;

/// Derive output file paths for all inputs, with collision avoidance.
///
/// Rules:
/// 1. Output filename = `{input_stem}.{format_extension}`
/// 2. If `output_dir` is Some, place in that directory
/// 3. If `output_dir` is None, place in current directory
/// 4. For collisions (same stem from different dirs), append `_{n}` suffix (n starts at 2)
#[must_use]
pub fn derive_output_paths(
    input_paths: &[PathBuf],
    format: OutputFormat,
    output_dir: Option<&Path>,
) -> Vec<(PathBuf, PathBuf)> {
    let ext = format.as_extension();
    let base_dir = output_dir.unwrap_or_else(|| Path::new("."));
    let mut claimed: HashSet<PathBuf> = HashSet::new();
    // Track next suffix per stem for O(1) collision resolution
    let mut next_suffix: HashMap<String, u32> = HashMap::new();
    let mut result = Vec::with_capacity(input_paths.len());

    for input in input_paths {
        let stem = input.file_stem().map_or_else(
            || input.to_string_lossy().into_owned(),
            |s| s.to_string_lossy().into_owned(),
        );

        let candidate = base_dir.join(format!("{stem}.{ext}"));

        let output_path = if claimed.contains(&candidate) {
            let n = next_suffix.entry(stem.clone()).or_insert(2);
            loop {
                let suffixed = base_dir.join(format!("{stem}_{n}.{ext}"));
                *n += 1;
                if !claimed.contains(&suffixed) {
                    break suffixed;
                }
            }
        } else {
            candidate
        };

        claimed.insert(output_path.clone());
        result.push((input.clone(), output_path));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // T007: derive_output_paths tests

    #[test]
    fn single_file_produces_stem_dot_json() {
        let inputs = vec![PathBuf::from("policy.md")];
        let pairs = derive_output_paths(&inputs, OutputFormat::Json, None);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].1, PathBuf::from("./policy.json"));
    }

    #[test]
    fn with_output_dir_places_in_dir() {
        let inputs = vec![PathBuf::from("policy.md")];
        let pairs = derive_output_paths(&inputs, OutputFormat::Json, Some(Path::new("/out")));
        assert_eq!(pairs[0].1, PathBuf::from("/out/policy.json"));
    }

    #[test]
    fn collision_appends_suffix_2() {
        let inputs = vec![PathBuf::from("/dir1/policy.md"), PathBuf::from("/dir2/policy.md")];
        let pairs = derive_output_paths(&inputs, OutputFormat::Json, Some(Path::new("/out")));
        assert_eq!(pairs[0].1, PathBuf::from("/out/policy.json"));
        assert_eq!(pairs[1].1, PathBuf::from("/out/policy_2.json"));
    }

    #[test]
    fn multiple_collisions_increment_suffix() {
        let inputs = vec![
            PathBuf::from("/a/policy.md"),
            PathBuf::from("/b/policy.md"),
            PathBuf::from("/c/policy.md"),
        ];
        let pairs = derive_output_paths(&inputs, OutputFormat::Json, Some(Path::new("/out")));
        assert_eq!(pairs[0].1, PathBuf::from("/out/policy.json"));
        assert_eq!(pairs[1].1, PathBuf::from("/out/policy_2.json"));
        assert_eq!(pairs[2].1, PathBuf::from("/out/policy_3.json"));
    }

    #[test]
    fn xml_format_uses_xml_extension() {
        let inputs = vec![PathBuf::from("policy.md")];
        let pairs = derive_output_paths(&inputs, OutputFormat::Xml, None);
        assert_eq!(pairs[0].1, PathBuf::from("./policy.xml"));
    }

    #[test]
    fn yaml_format_uses_yaml_extension() {
        let inputs = vec![PathBuf::from("policy.md")];
        let pairs = derive_output_paths(&inputs, OutputFormat::Yaml, None);
        assert_eq!(pairs[0].1, PathBuf::from("./policy.yaml"));
    }

    #[test]
    fn collision_suffix_skips_already_claimed_stem() {
        // policy.md → policy.json, policy_2.md → policy_2.json,
        // second policy.md collision must skip policy_2.json → policy_3.json
        let inputs = vec![
            PathBuf::from("/a/policy.md"),
            PathBuf::from("/b/policy_2.md"),
            PathBuf::from("/c/policy.md"),
        ];
        let pairs = derive_output_paths(&inputs, OutputFormat::Json, Some(Path::new("/out")));
        assert_eq!(pairs[0].1, PathBuf::from("/out/policy.json"));
        assert_eq!(pairs[1].1, PathBuf::from("/out/policy_2.json"));
        assert_eq!(pairs[2].1, PathBuf::from("/out/policy_3.json"));
    }
}
