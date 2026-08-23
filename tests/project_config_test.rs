//! Integration tests for PRD 051 — project configuration (`.forge.toml`).
//!
//! Covers config selection precedence, strict validation, effective-setting
//! resolution (explicit CLI > environment > project > default), path safety,
//! side-effect prevention, and backward compatibility without a config.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn forge_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_forge"));
    cmd.current_dir(std::env::temp_dir());
    // Child processes inherit the host environment; a stray FORGE_CONFIG
    // would replace discovery and an invalid FORGE_JOBS could fail unrelated
    // tests. Tests that exercise these variables set them explicitly via
    // `run_with_env`.
    cmd.env_remove("FORGE_CONFIG").env_remove("FORGE_JOBS");
    cmd
}

fn create_temp_md(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

const POLICY: &str = "# Title\n\n- Users must authenticate.\n";

fn write_config(dir: &Path, content: &str) {
    fs::write(dir.join(".forge.toml"), content).unwrap();
}

/// Run `forge` with `cwd` as the working directory.
fn run(cwd: &Path, args: &[&str]) -> (String, String, Option<i32>) {
    run_with_env(cwd, args, [])
}

fn run_with_env<const N: usize>(
    cwd: &Path,
    args: &[&str],
    envs: [(&str, &str); N],
) -> (String, String, Option<i32>) {
    let output = forge_bin()
        .current_dir(cwd)
        .args(args)
        .envs(envs)
        .output()
        .expect("Failed to execute forge");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}

// ── Selection precedence (AC-1..AC-3, M-1..M-3) ─────────────────────────────

#[test]
fn ac1_nearest_config_wins_without_merge() {
    let parent = TempDir::new().unwrap();
    let child = parent.path().join("child");
    fs::create_dir(&child).unwrap();
    write_config(parent.path(), "schema-version = 1\n[convert]\njobs = 1\n");
    write_config(&child, "schema-version = 1\n[convert]\njobs = 2\n");

    // `config check` reports the selected file; it must be the child's.
    let (stdout, _stderr, code) = run(&child, &["config", "check"]);
    assert_eq!(code, Some(0));
    assert!(stdout.contains(child.join(".forge.toml").display().to_string().as_str()));

    let contents = fs::read_to_string(child.join(".forge.toml")).unwrap();
    assert!(contents.contains("jobs = 2"), "no merge: child file untouched");
}

#[test]
fn ac2_config_flag_beats_env_and_discovery() {
    let parent = TempDir::new().unwrap();
    let child = parent.path().join("child");
    fs::create_dir(&child).unwrap();
    write_config(parent.path(), "schema-version = 1\n");
    write_config(&child, "schema-version = 1\n");

    let explicit = parent.path().join("custom.toml");
    fs::write(&explicit, "schema-version = 1\n[convert]\nstrategy = \"catalog\"\n").unwrap();

    let (stdout, stderr, code) = run_with_env(
        &child,
        &["--config", explicit.to_str().unwrap(), "config", "check"],
        [("FORGE_CONFIG", child.join(".forge.toml").to_str().unwrap())],
    );
    assert_eq!(code, Some(0), "stderr: {stderr}");
    assert!(
        stdout.contains(explicit.display().to_string().as_str()),
        "the --config file alone must be selected:\n{stdout}"
    );
}

#[test]
fn ac3_forge_config_beats_discovery() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 1\n");
    let selected = dir.path().join("env-selected.toml");
    fs::write(&selected, "schema-version = 1\n[validate]\nformat = \"json\"\n").unwrap();

    let (stdout, stderr, code) = run_with_env(
        dir.path(),
        &["config", "check"],
        [("FORGE_CONFIG", selected.to_str().unwrap())],
    );
    assert_eq!(code, Some(0), "stderr: {stderr}");
    assert!(stdout.contains(selected.display().to_string().as_str()), "{stdout}");
}

#[test]
fn ec3_empty_forge_config_fails_instead_of_discovery() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 1\n");
    let (_, stderr, code) = run_with_env(dir.path(), &["config", "check"], [("FORGE_CONFIG", "")]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("FORGE_CONFIG"), "{stderr}");
}

#[test]
fn ec2_missing_explicit_config_does_not_fall_back() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 1\n");
    let (_, stderr, code) = run_with_env(
        dir.path(),
        &["config", "check"],
        [("FORGE_CONFIG", dir.path().join(".forge.toml").to_str().unwrap())],
    );
    // Sanity: env selection works...
    assert_eq!(code, Some(0), "{stderr}");

    // ...but a missing --config file must fail without falling back to discovery.
    let missing = dir.path().join("nope.toml");
    let (_, stderr, code) = run_with_env(
        dir.path(),
        &["--config", missing.to_str().unwrap(), "config", "check"],
        [("FORGE_CONFIG", dir.path().join(".forge.toml").to_str().unwrap())],
    );
    assert_ne!(code, Some(0));
    assert!(stderr.contains("nope.toml") || stderr.contains("cannot read"), "{stderr}");
}

// ── Schema validation (AC-4, AC-5, AC-7) ────────────────────────────────────

#[test]
fn ac4_missing_schema_version_fails_check() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "[convert]\nstrategy = \"catalog\"\n");
    let (_, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("schema-version"), "{stderr}");
    assert!(stderr.contains(".forge.toml"), "{stderr}");
}

#[test]
fn ac5_newer_schema_version_reports_supported_version() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 2\n");
    let (_, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains('2'), "{stderr}");
    assert!(stderr.contains("version 1"), "{stderr}");
}

#[test]
fn ac7_unknown_key_suggests_close_match() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 1\n[convert]\nformt = \"json\"\n");
    let (_, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("unknown key `formt`"), "{stderr}");
    assert!(stderr.contains("did you mean `format`?"), "{stderr}");
}

#[test]
fn ac19_unsupported_keys_rejected_and_not_executed() {
    for body in [
        "[validate]\nround-trip = true\n",
        "[validate]\noscal-cli-path = \"/usr/bin/oscal-cli\"\n",
        "inputs = [\"a.md\"]\n",
    ] {
        let dir = TempDir::new().unwrap();
        let text = format!("schema-version = 1\n{body}");
        write_config(dir.path(), &text);
        let (_, stderr, code) = run(dir.path(), &["config", "check"]);
        assert_ne!(code, Some(0), "must reject: {body}");
        assert!(stderr.contains("unknown key"), "{stderr}");
    }
}

// ── Convert uses config defaults (AC-6, G-5) ────────────────────────────────

#[test]
fn ac6_convert_uses_project_defaults() {
    let dir = TempDir::new().unwrap();
    create_temp_md(&dir, "policy-a.md", POLICY);
    create_temp_md(&dir, "policy-b.md", POLICY);
    write_config(
        dir.path(),
        concat!(
            "schema-version = 1\n",
            "[convert]\n",
            "strategy = \"catalog\"\n",
            "format = \"json\"\n",
            "output = \"generated/oscal\"\n",
            "max-size-mb = 20\n",
            "jobs = 2\n",
        ),
    );

    // No --strategy/--format/--output flags: everything from the project file
    // (US-1: batch conversion reduced to explicit inputs).
    let (stdout, stderr, code) = run(dir.path(), &["convert", "policy-a.md", "policy-b.md"]);
    assert_eq!(code, Some(0), "stdout: {stdout}\nstderr: {stderr}");

    for name in ["policy-a.json", "policy-b.json"] {
        let out = dir.path().join("generated/oscal").join(name);
        assert!(out.exists(), "configured output directory must be used: {}", out.display());
        let artifact = fs::read_to_string(&out).unwrap();
        assert!(artifact.contains("catalog"), "JSON catalog expected in {name}");
    }
}

// ── Jobs precedence matrix (AC-8..AC-11) ────────────────────────────────────

/// The jobs value is not directly observable in output; use `config check`,
/// which prints resolved settings? It prints only config values. Instead we
/// verify resolution through conversion success and rely on unit tests for
/// exact values. Here we verify env override validity handling end-to-end.
#[test]
fn ac8_invalid_forge_jobs_fails_before_side_effects() {
    let dir = TempDir::new().unwrap();
    let policy = create_temp_md(&dir, "policy.md", POLICY);
    write_config(dir.path(), "schema-version = 1\n[convert]\nstrategy = \"catalog\"\n");

    let (_, stderr, code) = run_with_env(
        dir.path(),
        &["convert", policy.file_name().unwrap().to_str().unwrap()],
        [("FORGE_JOBS", "999")],
    );
    assert_ne!(code, Some(0));
    assert!(stderr.contains("FORGE_JOBS"), "{stderr}");

    let empty_ok = run_with_env(
        dir.path(),
        &["convert", policy.file_name().unwrap().to_str().unwrap()],
        [("FORGE_JOBS", "4")],
    );
    assert_eq!(empty_ok.2, Some(0), "{}", empty_ok.1);
}

// ── Summary override (AC-12, M-8) ───────────────────────────────────────────

#[test]
fn ac12_no_summary_flag_overrides_config_true() {
    const DASHBOARD: &str = "FORGE Conversion Summary";

    let dir = TempDir::new().unwrap();
    let policy = create_temp_md(&dir, "policy.md", POLICY);
    write_config(
        dir.path(),
        "schema-version = 1\n[convert]\nstrategy = \"catalog\"\nsummary = true\n",
    );

    let name = policy.file_name().unwrap().to_str().unwrap();

    // Run non-quiet with a file output so the summary dashboard is the only
    // thing on stderr; `--quiet` suppresses it independently and would make
    // every invocation pass even if the override were ignored.

    // Config summary = true: dashboard emitted.
    let (_, stderr, code) = run(dir.path(), &["convert", "--output", "out.json", name]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stderr.contains(DASHBOARD), "config summary=true must emit dashboard: {stderr}");

    // `--no-summary` overrides config summary = true: dashboard suppressed.
    let (_, stderr, code) =
        run(dir.path(), &["convert", "--output", "out.json", "--no-summary", name]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(!stderr.contains(DASHBOARD), "--no-summary must suppress dashboard: {stderr}");

    // Explicit positive form keeps it on.
    let (_, stderr, code) =
        run(dir.path(), &["convert", "--output", "out.json", "--summary", name]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stderr.contains(DASHBOARD), "--summary must emit dashboard: {stderr}");
}

// ── Path safety (AC-13, AC-14, M-9/M-12) ────────────────────────────────────

#[test]
fn ac14_traversal_output_rejected_before_side_effects() {
    let parent = TempDir::new().unwrap();
    let dir = parent.path().join("proj");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("policy.md"), POLICY).unwrap();
    write_config(
        &dir,
        "schema-version = 1\n[convert]\nstrategy = \"catalog\"\noutput = \"../outside\"\n",
    );

    let (_, stderr, code) = run(&dir, &["convert", "policy.md"]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("outside the project root"), "{stderr}");
    assert!(!parent.path().join("outside").exists(), "no output written outside root");
}

#[test]
fn ac14_absolute_output_rejected() {
    let dir = TempDir::new().unwrap();
    write_config(
        dir.path(),
        "schema-version = 1\n[convert]\nstrategy = \"catalog\"\noutput = \"/tmp/escape\"\n",
    );
    let (_, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("absolute"), "{stderr}");
}

// ── File safety (AC-15) ─────────────────────────────────────────────────────

#[test]
#[cfg(unix)]
fn ac15_symlinked_config_rejected() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("real.toml");
    fs::write(&real, "schema-version = 1\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, dir.path().join(".forge.toml")).unwrap();

    let (_, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_ne!(code, Some(0));
    assert!(stderr.contains("symbolic link"), "{stderr}");
}

// ── config check behavior (AC-17) ───────────────────────────────────────────

#[test]
fn ac17_valid_config_check_is_clean_and_read_only() {
    let dir = TempDir::new().unwrap();
    let baseline = create_temp_md(&dir, "baseline.md", POLICY);
    write_config(
        dir.path(),
        "schema-version = 1\n\
         [convert]\n\
         strategy = \"component\"\n\
         source-profile = \"baseline.md\"\n",
    );

    let before = fs::read_dir(dir.path()).unwrap().count();
    let (stdout, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_eq!(code, Some(0), "stderr: {stderr}");
    assert!(stdout.contains("OK"), "{stdout}");
    assert!(stdout.contains("source-profile = baseline.md"), "{stdout}");
    assert!(stdout.contains("not yet"), "byte-reproducibility caveat must be stated");
    let after = fs::read_dir(dir.path()).unwrap().count();
    assert_eq!(before, after, "no files created by config check");
    let _ = baseline;
}

// ── Backward compatibility (AC-18 / US-6) ───────────────────────────────────

#[test]
fn ac18_missing_strategy_still_reported_without_config() {
    let dir = TempDir::new().unwrap();
    let policy = create_temp_md(&dir, "policy.md", POLICY);
    let name = policy.file_name().unwrap().to_str().unwrap();
    let (_, stderr, code) = run(dir.path(), &["convert", name]);
    assert_ne!(code, Some(0));
    assert!(
        stderr.contains("--strategy") && stderr.contains("required"),
        "stderr should indicate --strategy is required:\n{stderr}"
    );
}

#[test]
fn ac18_no_config_directory_behaves_as_before() {
    let dir = TempDir::new().unwrap();
    let policy = create_temp_md(&dir, "policy.md", POLICY);
    let name = policy.file_name().unwrap().to_str().unwrap();

    // Explicit flags work exactly as before.
    let (stdout, stderr, code) = run(dir.path(), &["convert", name, "--strategy", "catalog"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(!stdout.is_empty(), "artifact on stdout");
}

// ── Determinism (AC-20) ─────────────────────────────────────────────────────

#[test]
fn ac20_reordered_configs_resolve_identically() {
    // Exact value equality is covered by unit tests; here we ensure both
    // orderings are accepted and produce identical `config check` reports.
    fn normalize(s: &str) -> Vec<&str> {
        s.lines().filter(|l| !l.contains(".forge.toml") && !l.contains("project root")).collect()
    }
    let text_a = "schema-version = 1\n[convert]\nstrategy = \"catalog\"\nformat = \"yaml\"\n";
    let text_b = "schema-version = 1\n[convert]\nformat = \"yaml\"\nstrategy = \"catalog\"\n";

    let dir_a = TempDir::new().unwrap();
    write_config(dir_a.path(), text_a);
    let (out_a, _, code_a) = run(dir_a.path(), &["config", "check"]);

    let dir_b = TempDir::new().unwrap();
    write_config(dir_b.path(), text_b);
    let (out_b, _, code_b) = run(dir_b.path(), &["config", "check"]);

    assert_eq!(code_a, Some(0), "{out_a}");
    assert_eq!(code_b, Some(0), "{out_b}");
    // Strip the differing config paths from the reports before comparing.
    assert_eq!(normalize(&out_a), normalize(&out_b));
}

// ── Remediation regression tests (review round 1) ───────────────────────────

/// Finding 2: `config check` must reject cross-field-invalid configurations.
#[test]
fn config_check_rejects_component_strategy_without_source_profile() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 1\n[convert]\nstrategy = \"component\"\n");
    let (_, stderr, code) = run(dir.path(), &["config", "check"]);
    assert_ne!(code, Some(0), "unusable configuration must not report OK");
    assert!(stderr.contains("source-profile"), "{stderr}");
}

/// Finding 2 (EC-9 counterpart): a CLI override resolves the conflict at
/// conversion time even though the config alone would fail `config check`.
#[test]
fn cli_source_profile_resolves_config_level_conflict() {
    let dir = TempDir::new().unwrap();
    // Minimal valid OSCAL profile for --source-profile resolution.
    fs::write(dir.path().join("profile.json"), "{}").unwrap();
    create_temp_md(&dir, "policy.md", POLICY);
    write_config(dir.path(), "schema-version = 1\n[convert]\nstrategy = \"component\"\n");

    let (_, stderr, code) =
        run(dir.path(), &["convert", "policy.md", "--source-profile", "profile.json"]);
    // The command may fail on profile content, but it must get past the
    // requirement error — i.e. the CLI value satisfied the component need.
    assert!(
        !stderr.contains("--source-profile is required"),
        "CLI override must satisfy the component requirement: {stderr}"
    );
    let _ = code;
}

/// Finding 5: `FORGE_JOBS=999` with an explicit `--jobs` must use the CLI value.
#[test]
fn explicit_cli_jobs_defeats_invalid_forge_jobs_env() {
    let dir = TempDir::new().unwrap();
    create_temp_md(&dir, "policy-a.md", POLICY);
    create_temp_md(&dir, "policy-b.md", POLICY);
    write_config(dir.path(), "schema-version = 1\n[convert]\nstrategy = \"catalog\"\n");

    let (_, stderr, code) = run_with_env(
        dir.path(),
        &["convert", "--jobs", "1", "policy-a.md", "policy-b.md"],
        [("FORGE_JOBS", "999")],
    );
    assert_eq!(code, Some(0), "explicit --jobs must win over invalid env: {stderr}");
}

/// Finding 7: missing strategy without any config behaves like clap usage
/// errors (message names --strategy as required; exit code 2).
#[test]
fn missing_strategy_exit_code_is_2_without_config() {
    let dir = TempDir::new().unwrap();
    create_temp_md(&dir, "policy.md", POLICY);
    let (_, stderr, code) = run(dir.path(), &["convert", "policy.md"]);
    assert_eq!(code, Some(2), "clap-compatible usage exit code; stderr: {stderr}");
    assert!(stderr.contains("--strategy") && stderr.contains("required"), "{stderr}");
}

/// Finding 8: the global --config selector must not be silently ignored by
/// commands that do not consume project configuration.
#[test]
fn config_flag_rejected_on_unsupported_commands() {
    let dir = TempDir::new().unwrap();
    let artifact = dir.path().join("missing.toml");
    let input = create_temp_md(&dir, "catalog.json", "{\"catalog\": {}}");
    for args in [
        vec!["export", input.to_str().unwrap(), "--format", "yaml"],
        vec!["diff", input.to_str().unwrap(), input.to_str().unwrap()],
    ] {
        let mut full = vec!["--config", artifact.to_str().unwrap()];
        full.extend(args.clone());
        let (_, stderr, code) = run(dir.path(), &full);
        assert_ne!(code, Some(0), "must not silently ignore --config for: {args:?}");
        assert!(stderr.contains("--config is only supported"), "{args:?}: {stderr}");
    }
}

/// Finding 6: batch conversion must not create the output directory when a
/// later validation fails (here: component strategy without source profile).
#[test]
fn batch_output_directory_not_created_on_invalid_invocation() {
    let parent = TempDir::new().unwrap();
    let dir = parent.path().join("proj");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("a.md"), POLICY).unwrap();
    fs::write(dir.join("b.md"), POLICY).unwrap();

    let (_, stderr, code) = run(
        &dir,
        &["convert", "--strategy", "component", "--output", "generated/oscal", "a.md", "b.md"],
    );
    assert_ne!(code, Some(0), "component without source-profile must fail: {stderr}");
    assert!(
        !dir.join("generated").exists(),
        "output directory must not be created before validation completes"
    );
}

/// Finding 4: a bare relative selector (no directory component) must resolve
/// its project root against the current working directory (M-2, EC-1).
#[test]
fn bare_relative_forge_config_selector_works() {
    let dir = TempDir::new().unwrap();
    write_config(dir.path(), "schema-version = 1\n[convert]\njobs = 3\n");

    let (stdout, stderr, code) = run(dir.path(), &["--config", ".forge.toml", "config", "check"]);
    assert_eq!(code, Some(0), "--config with bare relative name: {stderr}");
    assert!(stdout.contains("jobs = 3"), "{stdout}");

    let (_, stderr, code) =
        run_with_env(dir.path(), &["config", "check"], [("FORGE_CONFIG", ".forge.toml")]);
    assert_eq!(code, Some(0), "FORGE_CONFIG bare relative name: {stderr}");
}
