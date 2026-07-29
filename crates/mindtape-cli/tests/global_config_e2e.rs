//! Proof that the e2e harness keeps the machine's own `mt` configuration out
//! of the suite — and that the layer it hides is really there.
//!
//! `mt` merges a global `defaults.toml` (`dirs::config_dir()/mindtape/`) under
//! every project config. That is a supported layer, so the binary honours it;
//! what must not happen is a *test* honouring it, because then the suite
//! answers a question about the developer's home directory. One test here
//! shows `support::mt` sees no such layer, the other shows the loader still
//! reads one when it is genuinely present — without the second, the first
//! would keep passing if the layer quietly stopped working altogether.
//!
//! Both halves are staged through `$MINDTAPE_CONFIG_HOME`
//! (`mindtape::config_home`), the one override that lands on every platform:
//! `dirs` reads `XDG_CONFIG_HOME`/`HOME` on Unix but a known-folder API on
//! Windows, which no environment variable can move.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

/// `mt config show --origin` for `dir`, one line per merged layer. `config_home`
/// restores a global config directory the harness spawn otherwise points at
/// nothing.
fn origin(dir: &Path, config_home: Option<&Path>) -> String {
    let mut cmd = support::mt(dir);
    if let Some(path) = config_home {
        cmd.env(mindtape::CONFIG_HOME_ENV, path);
    }
    let output = cmd
        .args(["config", "show", "--origin"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

/// A `mindtape/defaults.toml` under a fresh config-home directory, standing in
/// for the one a developer keeps on their own machine.
fn config_home_with_defaults(body: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mindtape = dir.path().join("mindtape");
    std::fs::create_dir_all(&mindtape).unwrap();
    std::fs::write(mindtape.join("defaults.toml"), body).unwrap();
    dir
}

/// The harness spawn reports no `xdg:` layer, whatever the machine holds.
#[test]
fn the_harness_spawn_merges_no_global_defaults() {
    let project = support::scaffold_project("[tasks]\nroots = [\"tasks\"]\n");

    let layers = origin(project.path(), None);

    assert!(
        !layers.contains("xdg:"),
        "an e2e spawn must not merge the machine's global defaults: {layers}"
    );
}

/// Point the same spawn at a config home that does hold a `defaults.toml` and
/// the layer appears — so the test above is isolation working, not the layer
/// being dead.
#[test]
fn a_global_defaults_file_does_reach_the_loader_when_present() {
    let project = support::scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    let config_home = config_home_with_defaults("[ls]\nhyperlinks = \"always\"\n");

    let layers = origin(project.path(), Some(config_home.path()));

    assert!(
        layers.contains("xdg:"),
        "a global defaults.toml that exists must merge: {layers}"
    );
}

/// `$MINDTAPE_CONFIG_HOME` outranks the platform lookup: with `XDG_CONFIG_HOME`
/// aimed at a directory that holds a `defaults.toml`, an override pointing
/// elsewhere still hides it. The Unix-only half of the seam would report the
/// layer here.
#[test]
fn the_override_outranks_the_platform_config_lookup() {
    let project = support::scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    let xdg = config_home_with_defaults("[ls]\nhyperlinks = \"always\"\n");
    let empty = tempfile::tempdir().unwrap();

    let mut cmd = support::mt(project.path());
    cmd.env("XDG_CONFIG_HOME", xdg.path())
        .env(mindtape::CONFIG_HOME_ENV, empty.path());
    let output = cmd
        .args(["config", "show", "--origin"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let layers = String::from_utf8(output).unwrap();

    assert!(
        !layers.contains("xdg:"),
        "MINDTAPE_CONFIG_HOME must decide the global layer, not XDG: {layers}"
    );
}
