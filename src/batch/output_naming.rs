use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::cli::OutputFormat;

/// Derive output file paths for all inputs, with collision avoidance.
///
/// Rules:
/// 1. Output filename = `{input_stem}.{format_extension}`
/// 2. If `output_dir` is Some, place in that directory
/// 3. If `output_dir` is None, place in current directory
/// 4. For collisions (same stem from different dirs, or a file already on
///    disk), append `_{n}` suffix (n starts at 2)
/// 5. An output equal to its own input path (same file, `output_dir` is None)
///    is always suffixed so conversion never overwrites its source (F0286)
/// 6. Claimed generated paths compare ASCII case-insensitively to avoid collisions
///    on common case-insensitive filesystems (F0287).
#[must_use]
pub fn derive_output_paths(
    input_paths: &[PathBuf],
    format: OutputFormat,
    output_dir: Option<&Path>,
) -> Vec<(PathBuf, PathBuf)> {
    let ext = format.as_extension();
    let base_dir = output_dir.unwrap_or_else(|| Path::new("."));
    // ASCII-fold generated names because common target filesystems are case-insensitive.
    let mut claimed: HashSet<String> = HashSet::new();
    // Track next suffix per case-folded stem for O(1) collision resolution.
    let mut next_suffix: HashMap<String, u32> = HashMap::new();
    let mut result = Vec::with_capacity(input_paths.len());

    for input in input_paths {
        let stem = input.file_stem().map_or_else(
            || input.to_string_lossy().into_owned(),
            |s| s.to_string_lossy().into_owned(),
        );

        let candidate = base_dir.join(format!("{stem}.{ext}"));

        // A name is taken if minted in this call, already on disk, or equal
        // to the input it would overwrite (F0286).
        let taken = |path: &Path| {
            claimed.contains(&path.to_string_lossy().to_ascii_lowercase())
                || path.exists()
                || (output_dir.is_none() && same_file(input, path))
        };

        let output_path = if taken(&candidate) {
            let n = next_suffix.entry(stem.to_ascii_lowercase()).or_insert(2);
            loop {
                let suffixed = base_dir.join(format!("{stem}_{n}.{ext}"));
                *n += 1;
                if !taken(&suffixed) {
                    break suffixed;
                }
            }
        } else {
            candidate
        };

        claimed.insert(output_path.to_string_lossy().to_ascii_lowercase());
        result.push((input.clone(), output_path));
    }

    result
}

/// Cheap lexical identity probe used only for the self-clobber guard; a
/// canonicalizing comparison is unnecessary because `output_dir: None`
/// derives outputs in the current directory from the given input spelling.
fn same_file(input: &Path, output: &Path) -> bool {
    input == output
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
    fn ascii_case_variants_receive_distinct_outputs() {
        let inputs = vec![PathBuf::from("/one/policy.md"), PathBuf::from("/two/POLICY.md")];
        let pairs = derive_output_paths(&inputs, OutputFormat::Json, Some(Path::new("/out")));

        assert_eq!(pairs[0].1, PathBuf::from("/out/policy.json"));
        assert_eq!(pairs[1].1, PathBuf::from("/out/POLICY_2.json"));
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

    #[test]
    fn existing_output_on_disk_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("policy.md");
        let existing = dir.path().join("policy.json");
        std::fs::write(&input, "policy").unwrap();
        std::fs::write(&existing, "existing output").unwrap();

        let pairs = derive_output_paths(&[input], OutputFormat::Json, Some(dir.path()));

        assert_eq!(pairs[0].1, dir.path().join("policy_2.json"));
    }

    #[test]
    fn json_input_is_never_its_own_json_output() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("policy.json");
        std::fs::write(&input, "{}").unwrap();
        let pairs =
            derive_output_paths(std::slice::from_ref(&input), OutputFormat::Json, Some(dir.path()));

        assert_eq!(pairs[0].1, dir.path().join("policy_2.json"));
        assert_ne!(pairs[0].0, pairs[0].1);
    }
}
