#![allow(clippy::unwrap_used)]

use super::{PresetAction, PresetArgs, SaveArgs, render_ls, save};
use crate::cmd::init::preset::{self, DEFAULT_PRESET};
use mindtape::test_support::write_file;
use mindtape::test_support::{tempdir, tempdir_with_files};

/// A project fixture: `.mindtape/config.toml`, optionally with
/// `.mindtape/templates/*.typ`.
fn write_project(root: &std::path::Path, config: &str, templates: &[(&str, &str)]) {
    let dir = root.join(".mindtape");
    std::fs::create_dir_all(&dir).unwrap();
    write_file(&dir, "config.toml", config);
    if templates.is_empty() {
        return;
    }
    let template_dir = dir.join("templates");
    std::fs::create_dir_all(&template_dir).unwrap();
    for (file, body) in templates {
        write_file(&template_dir, file, body);
    }
}

/// Saving then resolving by name round-trips both the config and the
/// templates.
#[test]
fn save_then_resolve_round_trips_config_and_templates() {
    let project = tempdir();
    let user_dir = tempdir();
    write_project(
        project.path(),
        "[tasks]\nroots = [\"mine\"]\n",
        &[("task.typ", "// t"), ("other.typ", "// o")],
    );

    let dest = save(project.path(), user_dir.path(), "mine", false).unwrap();
    assert_eq!(dest, user_dir.path().join("mine"));

    let resolved = preset::resolve("mine", Some(user_dir.path())).unwrap();
    assert!(resolved.config.contains("mine"), "{}", resolved.config);
    assert_eq!(
        resolved
            .templates
            .iter()
            .map(|(file, _)| file.as_str())
            .collect::<Vec<_>>(),
        vec!["other.typ", "task.typ"]
    );
}

/// Saving from a project with no `.mindtape/templates` directory works, and
/// writes just the config.
#[test]
fn save_with_no_templates_directory_writes_config_only() {
    let project = tempdir();
    let user_dir = tempdir();
    write_project(project.path(), "[tasks]\nroots = [\"bare\"]\n", &[]);

    let dest = save(project.path(), user_dir.path(), "bare", false).unwrap();
    assert!(dest.join("config.toml").is_file());
    assert!(!dest.join("templates").exists());

    let resolved = preset::resolve("bare", Some(user_dir.path())).unwrap();
    assert!(resolved.templates.is_empty());
}

/// A project and user preset dir with an initial `"dup"` preset (`roots =
/// ["a"]`) already saved, for the two clobber tests below to continue from.
fn saved_dup() -> (tempfile::TempDir, tempfile::TempDir) {
    let project = tempdir();
    let user_dir = tempdir();
    write_project(project.path(), "[tasks]\nroots = [\"a\"]\n", &[]);
    save(project.path(), user_dir.path(), "dup", false).unwrap();
    (project, user_dir)
}

/// A second save of the same name without `--force` refuses and names what
/// already exists.
#[test]
fn save_refuses_to_clobber_without_force() {
    let (project, user_dir) = saved_dup();

    write_project(project.path(), "[tasks]\nroots = [\"b\"]\n", &[]);
    let err = save(project.path(), user_dir.path(), "dup", false)
        .unwrap_err()
        .to_string();
    assert!(err.contains("dup"), "{err}");
    assert!(err.contains("already exists"), "{err}");

    // The original save is untouched.
    let resolved = preset::resolve("dup", Some(user_dir.path())).unwrap();
    assert!(resolved.config.contains('a'), "{}", resolved.config);
}

/// `--force` overwrites an existing user preset.
#[test]
fn save_with_force_overwrites() {
    let (project, user_dir) = saved_dup();

    write_project(project.path(), "[tasks]\nroots = [\"b\"]\n", &[]);
    save(project.path(), user_dir.path(), "dup", true).unwrap();

    let resolved = preset::resolve("dup", Some(user_dir.path())).unwrap();
    assert!(resolved.config.contains('b'), "{}", resolved.config);
}

/// `--force` over a flat `<name>.toml` layout replaces it with the nested
/// layout `save` always writes, so the new save is not silently shadowed by
/// the flat file it superseded.
#[test]
fn save_with_force_replaces_a_flat_layout() {
    let project = tempdir();
    let user_dir = tempdir_with_files(&[("flat.toml", "[tasks]\nroots = [\"old\"]\n")]);
    write_project(project.path(), "[tasks]\nroots = [\"new\"]\n", &[]);

    save(project.path(), user_dir.path(), "flat", true).unwrap();

    assert!(!user_dir.path().join("flat.toml").exists());
    let resolved = preset::resolve("flat", Some(user_dir.path())).unwrap();
    assert!(resolved.config.contains("new"), "{}", resolved.config);
}

/// `ls` marks a user preset that shadows a built-in of the same name, and the
/// default built-in.
#[test]
fn ls_marks_a_shadowed_builtin_and_the_default() {
    let user_dir = tempdir_with_files(&[(
        &format!("{DEFAULT_PRESET}.toml"),
        "[tasks]\nroots = [\"mine\"]\n",
    )]);

    let out = render_ls(Some(user_dir.path())).unwrap();
    let builtin_line = out
        .lines()
        .find(|line| line.trim_start().starts_with(DEFAULT_PRESET))
        .unwrap();
    assert!(builtin_line.contains("default"), "{builtin_line}");
    assert!(
        builtin_line.contains("shadowed by user preset"),
        "{builtin_line}"
    );
    assert!(out.contains("shadows built-in"), "{out}");
}

/// `ls` with no user preset directory still lists the built-ins, unmarked.
#[test]
fn ls_with_no_user_presets_lists_only_builtins() {
    let out = render_ls(None).unwrap();
    assert!(out.contains(DEFAULT_PRESET));
    assert!(!out.contains("user presets"));
    assert!(!out.contains("shadow"));
}

/// `ls` over an empty (but existing) user preset directory reports no user
/// presets, same as a missing one.
#[test]
fn ls_with_empty_user_preset_dir_lists_only_builtins() {
    let user_dir = tempdir();
    let out = render_ls(Some(user_dir.path())).unwrap();
    assert!(!out.contains("user presets"));
}

/// A user preset that matches no built-in and is not the default is marked
/// with neither marker. Pins both `==` comparisons in `render_ls`: inverting
/// either spuriously annotates a novel name.
#[test]
fn ls_leaves_a_novel_user_preset_unmarked() {
    let user_dir =
        tempdir_with_files(&[("zzz-not-a-builtin.toml", "[tasks]\nroots = [\"mine\"]\n")]);

    let out = render_ls(Some(user_dir.path())).unwrap();
    let line = out
        .lines()
        .find(|line| line.contains("zzz-not-a-builtin"))
        .unwrap();
    assert!(!line.contains("shadows built-in"), "{line}");
    assert!(!line.contains("default"), "{line}");
}

/// `run` dispatch tests: `save`/`render_ls` above cover the logic, these cover
/// the `PresetArgs` -> `run` wiring itself (dispatch, `resolve_root`,
/// `println!`/`print!`), which is otherwise untested. `resolve_root` reads the
/// process cwd and `cmd::init::preset::user_preset_dir` reads `$MINDTAPE_CONFIG_HOME`, so
/// each case re-execs the test binary rather than mutating the shared
/// process's cwd/env.
const RUN_CHILD_ENV: &str = "_MT_PRESET_RUN_CHILD";

fn run_child_output(name: &str, cwd: &std::path::Path, config_home: &std::path::Path) -> String {
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .env(RUN_CHILD_ENV, "1")
        .env(mindtape::CONFIG_HOME_ENV, config_home)
        .current_dir(cwd)
        .args([name, "--nocapture"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `run` with the `Ls` action delegates to `render_ls` and prints its result.
#[test]
fn run_ls_prints_builtin_presets() {
    if std::env::var(RUN_CHILD_ENV).is_err() {
        let empty_home = tempdir();
        let cwd = tempdir();
        let stdout = run_child_output(
            "run_ls_prints_builtin_presets",
            cwd.path(),
            empty_home.path(),
        );
        assert!(stdout.contains("built-in presets:"), "{stdout}");
        return;
    }
    let args = PresetArgs {
        action: PresetAction::Ls { json: false },
    };
    super::run(&args).unwrap();
}

/// `run` with the `Save` action resolves the project root from the cwd,
/// writes the user preset under `$MINDTAPE_CONFIG_HOME`, and prints the
/// destination.
#[test]
fn run_save_writes_preset_and_prints_destination() {
    if std::env::var(RUN_CHILD_ENV).is_err() {
        let config_home = tempdir();
        let project = tempdir();
        write_project(project.path(), "[tasks]\nroots = [\"mine\"]\n", &[]);

        let stdout = run_child_output(
            "run_save_writes_preset_and_prints_destination",
            project.path(),
            config_home.path(),
        );
        assert!(stdout.contains("saved preset"), "{stdout}");
        let written = config_home
            .path()
            .join("mindtape")
            .join("presets")
            .join("mine")
            .join("config.toml");
        assert!(written.is_file(), "{}", written.display());
        return;
    }
    let args = PresetArgs {
        action: PresetAction::Save(SaveArgs {
            name: "mine".to_owned(),
            force: false,
        }),
    };
    super::run(&args).unwrap();
}

/// `run` with the `Save` action outside a Mindtape project surfaces
/// `resolve_root`'s error rather than panicking or silently doing nothing.
#[test]
fn run_save_outside_project_fails() {
    if std::env::var(RUN_CHILD_ENV).is_err() {
        let config_home = tempdir();
        let cwd = tempdir();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .env(RUN_CHILD_ENV, "1")
            .env(mindtape::CONFIG_HOME_ENV, config_home.path())
            .current_dir(cwd.path())
            .args(["run_save_outside_project_fails", "--nocapture"])
            .output()
            .unwrap();
        assert!(
            !out.status.success(),
            "run() must fail outside a Mindtape project"
        );
        return;
    }
    let args = PresetArgs {
        action: PresetAction::Save(SaveArgs {
            name: "mine".to_owned(),
            force: false,
        }),
    };
    super::run(&args).unwrap();
}
