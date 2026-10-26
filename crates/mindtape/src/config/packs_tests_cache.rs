//! A pack from a declared source: `acme:<pack>` resolves to `<pack>.toml` or
//! `<pack>/pack.toml` in the cached tree the lock's `sha256` names, once the
//! tree hashes to that value. The cached tree is untrusted, so none of its
//! files resolve outside it.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use crate::config::layers::load_config_layered_in;
use crate::config::pack_registry::{PackOrigin, RuleOrigin};
use crate::config::pack_sources::hash;
use crate::config::{Config, ConfigError};
use crate::test_support::{tempdir, write_file};

const SOURCE: &str =
    "[pack-sources.acme]\ngit = \"https://example.com/acme/packs\"\nref = \"v1\"\n";

const RULE: &str = "[[tasks.rules]]\nname = \"acme-rule\"\nassert = 'kind == \"bug\"'\n";

/// A project whose config declares `acme` and extends `entry`, and a cache
/// root holding `files` as the tree the lock pins.
struct Fixture {
    _tmp: tempfile::TempDir,
    project: PathBuf,
    cache: PathBuf,
    tree: PathBuf,
}

/// Write `files` as acme's tree, hash it into the lock, and rename it to
/// `<cache>/<sha256>/`, as a fetch leaves it.
fn fixture(entry: &str, files: &[(&str, &str)]) -> Fixture {
    let tmp = tempdir();
    let project = tmp.path().join("project");
    let cache = tmp.path().join("cache");
    let staged = tmp.path().join("staged");
    std::fs::create_dir_all(&staged).unwrap();
    for (rel, text) in files {
        write_file(&staged, rel, text);
    }
    let sha = hash(&staged).unwrap();
    std::fs::create_dir_all(&cache).unwrap();
    let tree = cache.join(&sha);
    std::fs::rename(&staged, &tree).unwrap();
    let config = format!("extends = [{entry}]\n\n{SOURCE}");
    write_file(&project, ".mindtape/config.toml", config);
    let lock = format!(
        "[acme]\ngit = \"https://example.com/acme/packs\"\nref = \"v1\"\n\
         commit = \"3f2c1a9e0b7d4c55e1a2b3c4d5e6f708192a3b4c\"\nsha256 = \"{sha}\"\n"
    );
    write_file(&project, ".mindtape/packs.lock", lock);
    Fixture {
        _tmp: tmp,
        project,
        cache,
        tree,
    }
}

impl Fixture {
    /// Load the project against `cache`, with no ancestor or user layer.
    fn load_from(
        &self,
        cache: Option<&Path>,
    ) -> Result<(Config, Vec<crate::config::pack_registry::RuleOriginEntry>), ConfigError> {
        load_config_layered_in(&self.project, Some(&self.project), None, cache)
            .map(|(config, _, origins)| (config, origins))
    }

    fn load(
        &self,
    ) -> Result<(Config, Vec<crate::config::pack_registry::RuleOriginEntry>), ConfigError> {
        self.load_from(Some(&self.cache))
    }

    /// The message and hint of the refused load.
    fn refusal(&self) -> (String, Option<&'static str>) {
        refusal_of(self.load())
    }
}

fn refusal_of<T>(loaded: Result<T, ConfigError>) -> (String, Option<&'static str>) {
    match loaded {
        Ok(_) => panic!("expected the config to be refused"),
        Err(ConfigError::Invalid { message, hint, .. }) => (message, hint),
        Err(other) => panic!("expected an invalid config, got {other:?}"),
    }
}

#[test]
fn a_source_pack_file_resolves_from_the_cached_tree() {
    let fx = fixture("\"acme:commits\"", &[("commits.toml", RULE)]);

    let (config, origins) = fx.load().unwrap();

    assert_eq!(config.tasks.rules[0].name.as_deref(), Some("acme-rule"));
    assert_eq!(
        origins[0].origin,
        RuleOrigin::Pack(PackOrigin::Cached {
            name: "acme:commits".to_owned(),
            tree: fx.tree.clone(),
            dir: None,
        }),
    );
}

#[test]
fn a_source_pack_directory_reads_its_pack_toml_and_templates() {
    let files = [("team/pack.toml", RULE), ("team/templates/feat.typ", "")];
    let fx = fixture("\"acme:team\"", &files);

    let (config, origins) = fx.load().unwrap();

    let dir = fx.tree.join("team");
    assert_eq!(
        origins[0].origin,
        RuleOrigin::Pack(PackOrigin::Cached {
            name: "acme:team".to_owned(),
            tree: fx.tree.clone(),
            dir: Some(dir.clone()),
        }),
    );
    let templates = dir.join("templates").display().to_string();
    assert!(
        config.templates.dirs.contains(&templates),
        "{:?}",
        config.templates.dirs
    );
}

#[test]
fn a_tree_that_no_longer_hashes_to_the_lock_is_refused_naming_its_directory() {
    let fx = fixture("\"acme:commits\"", &[("commits.toml", RULE)]);
    std::fs::write(fx.tree.join("commits.toml"), "").unwrap();

    let (message, hint) = fx.refusal();

    let dir = crate::config::quote(&fx.tree.display().to_string());
    assert_eq!(
        message,
        format!("extends: pack source 'acme' does not match the lock: {dir}")
    );
    assert!(
        hint.is_some_and(|h| h.contains("mt packs install")),
        "{hint:?}"
    );
}

#[test]
fn a_file_added_to_the_cached_tree_is_refused_too() {
    let fx = fixture("\"acme:commits\"", &[("commits.toml", RULE)]);
    std::fs::write(fx.tree.join("extra.toml"), RULE).unwrap();

    let (message, _) = fx.refusal();

    assert!(message.contains("does not match the lock"), "{message}");
}

#[cfg(unix)]
#[test]
fn a_symlink_planted_in_the_cached_tree_is_refused() {
    let fx = fixture("\"acme:commits\"", &[("commits.toml", RULE)]);
    std::os::unix::fs::symlink("/etc/hostname", fx.tree.join("link")).unwrap();

    let (message, _) = fx.refusal();

    let dir = crate::config::quote(&fx.tree.display().to_string());
    assert_eq!(
        message,
        format!(
            "extends: pack source 'acme' does not match the lock: {dir}: \
             pack source holds a symlink: 'link'"
        )
    );
}

#[test]
fn a_source_missing_from_the_cache_is_not_installed() {
    let fx = fixture("\"acme:commits\"", &[("commits.toml", RULE)]);
    std::fs::remove_dir_all(&fx.tree).unwrap();

    let (message, hint) = fx.refusal();

    assert_eq!(message, "extends: pack source not installed: 'acme'");
    assert!(
        hint.is_some_and(|h| h.contains("mt packs install")),
        "{hint:?}"
    );
    let (none, _) = refusal_of(fx.load_from(None));
    assert_eq!(none, message);
}

#[test]
fn an_unknown_pack_lists_the_packs_the_tree_holds() {
    let files = [
        ("commits.toml", RULE),
        ("team/pack.toml", ""),
        ("notes/readme.md", ""),
        ("readme.md", ""),
    ];
    let fx = fixture("\"acme:comits\"", &files);

    let err = fx.load().unwrap_err();

    let ConfigError::Invalid {
        message, unknown, ..
    } = err
    else {
        panic!("expected an invalid config, got {err:?}");
    };
    assert_eq!(message, "extends: unknown pack: 'acme:comits'");
    let unknown = unknown.unwrap();
    assert_eq!(unknown.names, ["acme:commits", "acme:team"]);
    assert_eq!(unknown.nearest.as_deref(), Some("acme:commits"));
}

#[test]
fn a_pack_name_that_could_leave_the_tree_is_refused() {
    for name in ["..", ".", "", "a.b", "a:b"] {
        let fx = fixture(&format!("\"acme:{name}\""), &[("pack.toml", RULE)]);

        let (message, _) = fx.refusal();

        assert_eq!(
            message,
            format!("extends: a pack name takes letters, digits, '-' and '_': 'acme:{name}'"),
        );
    }
}

#[test]
fn a_pack_both_a_file_and_a_directory_is_ambiguous() {
    let files = [("team.toml", RULE), ("team/pack.toml", RULE)];
    let fx = fixture("\"acme:team\"", &files);

    let (message, _) = fx.refusal();

    assert!(
        message.starts_with("extends: ambiguous pack: "),
        "{message}"
    );
}

#[test]
fn a_cached_packs_dictionary_resolves_inside_its_tree() {
    let pack = "[dicts.verbs]\npath = \"../words.txt\"\n";
    let files = [("team/pack.toml", pack), ("words.txt", "fix\n")];
    let fx = fixture("\"acme:team\"", &files);

    let (config, _) = fx.load().unwrap();

    let path = config.dicts["verbs"].path.clone().unwrap();
    assert_eq!(PathBuf::from(path), fx.tree.join("words.txt"));
}

#[test]
fn a_cached_packs_dictionary_outside_its_tree_is_refused() {
    for path in ["../../outside.txt", "/etc/hostname"] {
        let pack = format!("[dicts.verbs]\npath = \"{path}\"\n");
        let fx = fixture("\"acme:commits\"", &[("commits.toml", &pack)]);
        write_file(fx.cache.parent().unwrap(), "outside.txt", "fix\n");

        let (message, _) = fx.refusal();

        assert_eq!(
            message,
            format!(
                "extends: pack 'acme:commits' names a file outside its source: dicts.verbs.path: '{path}'"
            ),
        );
    }
}

/// acme's `team` pack, whose rule calls `check-x` from `module`, with
/// `check-x` written at `file` in the tree.
fn check_pack(module: &str, file: &str) -> Fixture {
    let rule = format!(
        "[[tasks.rules]]\nname = \"x\"\n\
         check = {{ typst = \"{module}\", function = \"check-x\" }}\n"
    );
    let check = "#let check-x(task) = (\"from acme\",)\n";
    fixture("\"acme:team\"", &[("team/pack.toml", &rule), (file, check)])
}

#[test]
fn a_cached_packs_check_outside_its_directory_is_refused() {
    let fx = check_pack("../check.typ", "check.typ");

    let (message, _) = fx.refusal();

    assert_eq!(
        message,
        "tasks.rules[0].check: `../check.typ` is outside the pack's directory"
    );
}

#[test]
fn a_cached_packs_check_runs_from_its_directory() {
    let fx = check_pack("check.typ", "team/check.typ");

    let (config, _) = fx.load().unwrap();

    let spec = config.tasks.rules[0].check.as_ref().unwrap();
    assert_eq!(spec.root.as_deref(), Some(fx.tree.join("team").as_path()));
    let check = crate::rules::typst::CompiledCheck::new(spec, &fx.project, &config);
    let breaches = check.evaluate(&crate::test_support::bare_task("a"), &config);
    assert_eq!(breaches, ["from acme"]);
}

#[test]
fn only_the_project_layer_may_use_a_cached_source() {
    let fx = fixture("", &[("commits.toml", RULE)]);
    write_file(
        &fx.project,
        ".mindtape/defaults.toml",
        "extends = [\"acme:commits\"]\n",
    );

    let (message, _) = refusal_of(load_config_layered_in(
        &fx.project,
        fx.project.parent(),
        None,
        Some(&fx.cache),
    ));

    assert_eq!(
        message,
        "extends: only the project config may use pack source: 'acme'"
    );
}
