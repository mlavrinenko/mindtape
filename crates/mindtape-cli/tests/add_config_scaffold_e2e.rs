//! End-to-end tests for the config-derived values `add` splices into a task
//! when the project config is not the barebones default: a required priority
//! slot in its various shapes (raw int, real scale, strict framework), an
//! initial status derived from a custom legend, a required estimate's bounded
//! default, and the rule check `--print` shares with the write it previews.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

/// A config whose `[tasks.priority]` is required: the barebones template omits
/// priority, so `add` must splice it in.
const PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n";

#[test]
fn injects_config_required_priority_slot() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe` surfaces the spliced-in priority slot.
    let names = support::describe_slot_names(dir.path());
    assert!(names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling it writes a valid task.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "with-pri",
            "--set",
            "title=T",
            "--set",
            "priority=7",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-pri.typ")).unwrap();
    assert!(written.contains("priority: 7"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // Omitting a config-required field is the existing missing-slot hard error.
    support::mt(dir.path())
        .args(["add", "--name", "no-pri", "--set", "title=T"])
        .assert()
        .failure();
}

/// A config whose `[tasks.priority]` is real-scaled and required: the
/// barebones template omits priority, so `add` must splice in a `real`-kind
/// slot, not the `int` it would wrongly emit for an int-scaled field.
///
/// `bands = []` clears the embedded floor's int-contiguous bands (0-2/3-6/7-9),
/// which are non-contiguous under real semantics; a real-scale project either
/// defines real bands or drops them.
const REAL_PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"real\"\nmin = 0\nmax = 9\nrequired = true\nbands = []\n";

#[test]
fn injects_real_scale_priority_slot_and_round_trips_a_decimal() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        REAL_PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe=json` reports the spliced-in priority slot as `real`.
    assert_eq!(support::describe_slot_kind(dir.path(), "priority"), "real");

    // Filling it with a decimal writes a float literal without truncation.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "with-real-pri",
            "--set",
            "title=T",
            "--set",
            "priority=2.5",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-real-pri.typ")).unwrap();
    assert!(written.contains("priority: 2.5"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task with a real priority value passes `check`.
    support::mt(dir.path()).arg("check").assert().success();
}

/// A config whose priority is a required strict framework (`allow-manual =
/// false`): a scalar priority is invalid by construction, so `add` must splice
/// the framework call itself with one leaf slot per input — otherwise the
/// fallback template could never file a valid task in this project.
const STRICT_ICE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = \"!terminal\"\n\
     framework = \"ice\"\nallow-manual = false\n";

#[test]
fn injects_framework_leaf_slots_for_a_strict_framework_priority() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(dir.path().join(".mindtape/config.toml"), STRICT_ICE_CONFIG).unwrap();

    // `--describe` surfaces the framework leaves, not a scalar priority.
    let names = support::describe_slot_names(dir.path());
    for leaf in ["impact", "confidence", "ease"] {
        assert!(names.iter().any(|n| n == leaf), "{names:?}");
    }
    assert!(!names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling the leaves writes a valid framework priority.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "with-ice",
            "--set",
            "title=T",
            "--set",
            "impact=9",
            "--set",
            "confidence=0.9",
            "--set",
            "ease=6",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-ice.typ")).unwrap();
    assert!(written.contains("priority: framework(\"ice\""), "{written}");
    assert!(written.contains("impact: 9.0"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check` (the framework score lands in the scale).
    support::mt(dir.path()).arg("check").assert().success();

    // Omitting a leaf is the existing missing-slot hard error, before any write.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "no-ease",
            "--set",
            "title=T",
            "--set",
            "impact=9",
            "--set",
            "confidence=0.9",
        ])
        .assert()
        .failure();
    assert!(!dir.path().join("tasks/no-ease.typ").exists());
}

/// A config whose legend has no `"proposed"` entry at all — the initial
/// status `add` stamps must be derived from this legend, not a hardcoded
/// literal that would fail `check`.
const CUSTOM_LEGEND_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"todo\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

#[test]
fn derives_initial_status_from_a_custom_legend() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        CUSTOM_LEGEND_CONFIG,
    )
    .unwrap();

    support::mt(dir.path())
        .args(["add", "--name", "custom-legend", "--set", "title=T"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/custom-legend.typ")).unwrap();
    assert!(written.contains("status: todo("), "{written}");
    assert!(!written.contains("proposed"), "{written}");

    // The filed task passes `check` even though the legend has no `proposed`.
    support::mt(dir.path()).arg("check").assert().success();
}

/// A template's own `status:` names the status `add` files at: the name is
/// kept, the date restamped, and `--status` still overrides it.
#[test]
fn files_at_the_status_the_template_names() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        CUSTOM_LEGEND_CONFIG,
    )
    .unwrap();
    let templates = dir.path().join(".mindtape/templates");
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::write(
        templates.join("task.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: slot(\"title\"), status: done(2020, 1, 1))\n",
    )
    .unwrap();

    let add = |name: &str, extra: &[&str]| {
        support::mt(dir.path())
            .args(["add", "--name", name, "--set", "title=T"])
            .args(extra)
            .assert()
            .success();
        std::fs::read_to_string(dir.path().join(format!("tasks/{name}.typ"))).unwrap()
    };
    let written = add("named", &[]);
    assert!(written.contains("status: done("), "{written}");
    assert!(!written.contains("2020"), "{written}");
    let written = add("overridden", &["--status", "todo"]);
    assert!(written.contains("status: todo("), "{written}");
}

/// The real `.mindtape/templates/task.typ` this repo ships, so a drift in its
/// `estimate` slot default (e.g. drifting over the configured cap) fails
/// this test rather than only surfacing at dogfood `mt check` time.
const PRODUCTION_TEMPLATE: &str = include_str!("../../../.mindtape/templates/task.typ");

/// The template imports `../lib.typ` beside the tasks root. A stub that
/// accepts every verdict keeps this test about the estimate default, not about
/// which goals the repo currently holds.
const GOALS_STUB: &str =
    "#let helps(reason) = reason\n#let hurts(reason) = reason\n#let neutral = none\n";

/// Mirrors the dogfood config's `[tasks.estimate]`: required on every
/// non-terminal task, capped at 5 `src-files` by a rule, no minimum.
const ESTIMATE_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nframework = \"ice\"\n\
     [tasks.estimate]\nunits = [\"src-files\"]\nrequired = \"!terminal\"\n\
     [[tasks.rules]]\nassert = \"estimate.src-files <= 5\"\nseverity = \"error\"\n";

/// `estimate` required with no explicit `--set estimate=...`: the production
/// template's slot default must fill in a dict inside the configured cap,
/// not leave a task that fails `check` the moment it lands.
#[test]
fn add_with_no_explicit_estimate_defaults_to_a_valid_task() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        ESTIMATE_REQUIRED_CONFIG,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        PRODUCTION_TEMPLATE,
    )
    .unwrap();
    std::fs::write(dir.path().join("lib.typ"), GOALS_STUB).unwrap();

    support::mt(dir.path())
        .args(["add", "--name", "no-estimate", "--set", "title=No estimate"])
        .args(["--set", "problem=+ A step."])
        .args(["--set", "to-be=- An end state."])
        .args(["--set", "changes=- `a.rs`: a change."])
        .args(["--set", "out-of-scope=- Nothing."])
        .args(["--set", "dogfood=Not applicable."])
        .args(["--set", "kind=test"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/no-estimate.typ")).unwrap();
    assert!(written.contains("estimate: (src-files: 2)"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    support::mt(dir.path()).arg("check").assert().success();
}

/// A config whose declared estimate unit is `src-files` and whose one rule
/// caps it at 1, so a fill of 2 has something to refuse it on.
const ESTIMATE_UNIT_CONFIG: &str = r#"[tasks]
roots = ["tasks"]

[tasks.estimate]
units = ["src-files"]

[[tasks.rules]]
name = "cap"
assert = "estimate.src-files <= 1"
message = "keep it to one source file"
severity = "error"
"#;

/// A template filling an estimate of two source files — one over
/// [`ESTIMATE_UNIT_CONFIG`]'s cap.
const TWO_SOURCE_FILES_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title"),
  estimate: slot("estimate", kind: "text", default: "(src-files: 2)"),
)

== Summary
"#;

/// `--print` promises the task a write would produce, so it must run the same
/// pre-write check: a fill the config refuses fails the preview with the
/// write's error, writes and prints nothing. A preview that exits 0 while the
/// write it previews is refused is how the sweep shipped a retired estimate
/// unit — the dry run green, `--write` dead on its first finding.
#[test]
fn print_refuses_the_fill_a_write_would_refuse() {
    let dir =
        support::project_with_template_and_config(ESTIMATE_UNIT_CONFIG, TWO_SOURCE_FILES_TEMPLATE);

    let output = support::mt(dir.path())
        .args([
            "add",
            "--name",
            "over-cap",
            "--set",
            "title=Over cap",
            "--print",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !dir.path().join("tasks/over-cap.typ").exists(),
        "a preview never writes, refused or not"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("the task as filled:3: rules.cap: keep it to one source file"),
        "the preview must report the rule a write would report: {stderr}"
    );
    assert!(
        !stderr.contains("validation issue"),
        "the refusal is the breach, with no summary of its own: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "a refused fill must not print a preview: {output:?}"
    );

    // Under the cap the same preview prints, silently, and still writes nothing.
    let output = support::mt(dir.path())
        .args([
            "add",
            "--name",
            "under-cap",
            "--set",
            "title=Under cap",
            "--set",
            "estimate=(src-files: 1)",
            "--print",
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("estimate: (src-files: 1)"),
        "a valid fill still previews: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "a clean fill has nothing to warn about: {output:?}"
    );
    assert!(
        !dir.path().join("tasks/under-cap.typ").exists(),
        "--print never writes"
    );
}

/// A config whose `[tasks.assignee]` is required with a declared allowed list,
/// so the injected value must be a real actor. `$MINDTAPE_PRINCIPAL` fills it
/// when the caller says nothing.
const ASSIGNEE_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.assignee]\nallowed = [\"ada\", \"lin\"]\nrequired = true\n\
     [[actors]]\nname = \"ada\"\nkind = \"human\"\n\
     [[actors]]\nname = \"lin\"\nkind = \"human\"\n";

/// A config whose `[tasks.fields.kind]` is required with a default, so `add`
/// splices a choice slot and fills it from the config when the caller says
/// nothing.
const KIND_DEFAULT_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"feature\", \"bug\"]\nrequired = true\ndefault = \"feature\"\n";

/// Scaffold `config`, run `add --name <name> --set title=T` with `principal` in
/// `$MINDTAPE_PRINCIPAL` plus `extra` args, and return the child's stderr.
fn add_capture_stderr(
    dir: &Path,
    config: &str,
    name: &str,
    principal: &str,
    extra: &[&str],
) -> String {
    support::scaffold(dir);
    std::fs::write(dir.join(".mindtape/config.toml"), config).unwrap();
    let output = support::mt(dir)
        .env("MINDTAPE_PRINCIPAL", principal)
        .args(["add", "--name", name, "--set", "title=T"])
        .args(extra)
        .assert()
        .success()
        .get_output()
        .clone();
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The env-filled assignee gets a note naming the value, the config path and
/// the environment it came from — an agent that wanted a different actor can
/// see the default happened and how to override it.
#[test]
fn notes_the_env_filled_assignee() {
    let dir = tempfile::tempdir().unwrap();
    let stderr = add_capture_stderr(dir.path(), ASSIGNEE_REQUIRED_CONFIG, "noted", "ada", &[]);
    assert!(
        stderr.contains(
            "note: filled in from config: assignee = ada ([tasks.assignee] env MINDTAPE_PRINCIPAL)"
        ),
        "the env origin must be named: {stderr}"
    );
    assert!(
        stderr.contains("--set assignee=<value>"),
        "the override spelling must be named: {stderr}"
    );
    let written = std::fs::read_to_string(dir.path().join("tasks/noted.typ")).unwrap();
    assert!(written.contains("assignee: \"ada\""), "{written}");
}

/// A default-filled kind gets the same note, naming the config path and that
/// the value is a config default rather than something the caller typed.
#[test]
fn notes_the_default_filled_kind() {
    let dir = tempfile::tempdir().unwrap();
    let stderr = add_capture_stderr(dir.path(), KIND_DEFAULT_CONFIG, "defaulted", "ada", &[]);
    assert!(
        stderr
            .contains("note: filled in from config: kind = feature ([tasks.fields.kind] default)"),
        "the config default must be named: {stderr}"
    );
    let written = std::fs::read_to_string(dir.path().join("tasks/defaulted.typ")).unwrap();
    assert!(written.contains("kind: \"feature\""), "{written}");
}

/// A required `x-<ns>` field is spliced like `kind`: a choice over its
/// `@dict`-expanded values, set from the caller or filled from its default.
#[test]
fn splices_a_required_extension_field() {
    const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
         [dicts.machines]\nvalues = [\"alpha\", \"desk\"]\n\
         [tasks.fields.x-machine]\nvalues = [\"@machines\"]\nrequired = true\ndefault = \"alpha\"\n";
    let dir = tempfile::tempdir().unwrap();
    let stderr = add_capture_stderr(dir.path(), CONFIG, "filled", "ada", &[]);
    assert!(
        stderr.contains(
            "note: filled in from config: x-machine = alpha ([tasks.fields.x-machine] default)"
        ),
        "{stderr}"
    );
    support::mt(dir.path())
        .args(["add", "--name", "chosen", "--set", "title=T"])
        .args(["--set", "x-machine=desk"])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/chosen.typ")).unwrap();
    assert!(written.contains("x-machine: \"desk\""), "{written}");
}

/// A field the caller set itself draws no note, even when the config also
/// backs it with an environment.
#[test]
fn an_explicitly_set_field_draws_no_note() {
    let dir = tempfile::tempdir().unwrap();
    let stderr = add_capture_stderr(
        dir.path(),
        ASSIGNEE_REQUIRED_CONFIG,
        "chosen",
        "ada",
        &["--set", "assignee=lin"],
    );
    assert!(
        !stderr.contains("note: assignee"),
        "an explicit --set must suppress the note: {stderr}"
    );
    let written = std::fs::read_to_string(dir.path().join("tasks/chosen.typ")).unwrap();
    assert!(written.contains("assignee: \"lin\""), "{written}");
}
