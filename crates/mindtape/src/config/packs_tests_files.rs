//! The files a pack names, a `check.typst` module and a dictionary `path`,
//! resolve from the pack's own directory: the directory of a directory pack,
//! or the one holding a pack file. A layer's own paths still resolve from the
//! project root. The shared fixtures stay in the parent test modules.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use crate::config::Config;
use crate::config::layers::load_config_layered_from;
use crate::rules::typst::CompiledCheck;
use crate::test_support::{bare_task, tempdir_with_files};

/// The rule every pack here declares: `check-x` from the pack's `check.typ`.
const CHECK_RULE: &str = "[[tasks.rules]]\nname = \"x-is-valid\"\n\
     check = { typst = \"check.typ\", function = \"check-x\" }\n";

/// `check-x` returning one breach, `from <who>`.
fn check_x(who: &str) -> String {
    format!("#let check-x(task) = (\"from {who}\",)\n")
}

/// Load the project in `tmp`'s `project` folder, with `files` written under
/// `tmp` and `config` as its `config.toml`.
fn load(
    files: &[(&str, &str)],
    config: &str,
) -> (
    tempfile::TempDir,
    Result<Config, crate::config::ConfigError>,
) {
    let tmp = tempdir_with_files(files);
    let project = tmp.path().join("project");
    crate::test_support::write_file(&project, ".mindtape/config.toml", config);
    let loaded = load_config_layered_from(&project, Some(tmp.path()), None).map(|loaded| loaded.0);
    (tmp, loaded)
}

/// The one breach the first task rule's check gives a bare task, in the
/// project `files` and `config` load to.
fn first_breach(files: &[(&str, &str)], config: &str) -> String {
    let (tmp, loaded) = load(files, config);
    let config = loaded.unwrap();
    let spec = config.tasks.rules[0].check.as_ref().unwrap();
    let check = CompiledCheck::new(spec, &tmp.path().join("project"), &config);
    let mut breaches = check.evaluate(&bare_task("a"), &config);
    assert_eq!(breaches.len(), 1, "{breaches:?}");
    breaches.remove(0)
}

/// Where a pack sits and the `extends` entry that names it: a pack file, a
/// directory beside the project's config, and one outside the project.
const PACKS: [(&str, &str); 3] = [
    ("project/team", "./team/pack.toml"),
    ("project/team", "./team"),
    ("shared", "../shared"),
];

/// Run `verify` once per [`PACKS`] entry, with the files of a pack holding
/// `pack_toml` and `file`, whose text `content` gives for the entry, and the
/// project config extending it.
fn in_each_pack(
    pack_toml: &str,
    file: &str,
    content: impl Fn(&str) -> String,
    verify: impl Fn(&[(&str, &str)], &str, &str),
) {
    for (dir, entry) in PACKS {
        let toml = format!("{dir}/pack.toml");
        let named = format!("{dir}/{file}");
        let text = content(entry);
        let files = [(toml.as_str(), pack_toml), (named.as_str(), text.as_str())];
        verify(&files, &format!("extends = [\"{entry}\"]\n"), entry);
    }
}

#[test]
fn a_packs_check_file_resolves_from_the_packs_directory() {
    in_each_pack(CHECK_RULE, "check.typ", check_x, |files, config, entry| {
        assert_eq!(first_breach(files, config), format!("from {entry}"));
    });
}

#[test]
fn a_pack_check_file_imports_its_siblings_and_never_the_projects_files() {
    let pack = "#import \"helper.typ\": who\n#let check-x(task) = (who,)\n";
    let files = [
        ("project/team/pack.toml", CHECK_RULE),
        ("project/team/check.typ", pack),
        ("project/team/helper.typ", "#let who = \"the sibling\"\n"),
        ("project/helper.typ", "#let who = \"the project\"\n"),
    ];

    assert_eq!(
        first_breach(&files, "extends = [\"./team/pack.toml\"]\n"),
        "the sibling"
    );
}

#[test]
fn a_pack_check_file_cannot_reach_a_file_of_the_project() {
    let pack = "#import \"/secret.typ\": who\n#let check-x(task) = (who,)\n";
    let files = [
        ("project/team/pack.toml", CHECK_RULE),
        ("project/team/check.typ", pack),
        ("project/secret.typ", "#let who = \"the project\"\n"),
    ];

    let (_tmp, config) = load(&files, "extends = [\"./team/pack.toml\"]\n");

    let err = config.unwrap_err().to_string();
    assert!(err.contains("tasks.rules[0].check"), "{err}");
}

/// A pack and a project each holding a `check.typ` that says who it is from.
const TWO_CHECKS: [(&str, &str); 3] = [
    ("project/team/pack.toml", CHECK_RULE),
    (
        "project/team/check.typ",
        "#let check-x(task) = (\"from the pack\",)\n",
    ),
    (
        "project/check.typ",
        "#let check-x(task) = (\"from the project\",)\n",
    ),
];

#[test]
fn a_layers_own_check_file_still_resolves_from_the_project_root() {
    let own = CHECK_RULE.replace("x-is-valid", "own-x");

    let (tmp, config) = load(
        &TWO_CHECKS,
        &format!("extends = [\"./team/pack.toml\"]\n{own}"),
    );

    let config = config.unwrap();
    let spec = config.tasks.rules[1].check.as_ref().unwrap();
    let check = CompiledCheck::new(spec, &tmp.path().join("project"), &config);
    assert_eq!(
        check.evaluate(&bare_task("a"), &config),
        ["from the project"]
    );
}

#[test]
fn a_layer_rule_replacing_a_pack_rule_resolves_from_the_project_root() {
    let config = format!("extends = [\"./team/pack.toml\"]\n{CHECK_RULE}");

    assert_eq!(first_breach(&TWO_CHECKS, &config), "from the project");
}

/// The dictionary `words`, set by `[dicts.words] path = "words.txt"`.
const WORDS: &str = "[dicts.words]\npath = \"words.txt\"\n";

/// Whether the config `files` and `config` load to has `entry` in `words`.
fn words_hold(files: &[(&str, &str)], config: &str, entry: &str) -> bool {
    let (_tmp, loaded) = load(files, config);
    loaded.unwrap().dict_cache.dicts().contains("words", entry)
}

#[test]
fn a_packs_dict_path_resolves_from_the_packs_directory() {
    in_each_pack(
        WORDS,
        "words.txt",
        |_| "verb\n".to_owned(),
        |files, config, _| {
            assert!(words_hold(files, config, "verb"));
        },
    );
}

#[test]
fn a_layers_own_dict_path_still_resolves_from_the_project_root() {
    let files = [
        (
            "project/team/pack.toml",
            "[dicts.other]\nvalues = [\"x\"]\n",
        ),
        ("project/words.txt", "noun\n"),
    ];
    let config = format!("extends = [\"./team/pack.toml\"]\n{WORDS}");

    assert!(words_hold(&files, &config, "noun"));
}

#[test]
fn the_root_a_pack_rule_carries_is_its_pack_directory() {
    let files = [
        ("project/team/pack.toml", CHECK_RULE),
        ("project/team/check.typ", &check_x("the pack")),
    ];

    let (tmp, config) = load(&files, "extends = [\"./team/pack.toml\"]\n");

    let check = config.unwrap().tasks.rules[0].check.clone().unwrap();
    let team: PathBuf = tmp.path().join("project").join("team");
    assert_eq!(check.root.as_deref(), Some(team.as_path()));
}

/// The message a pack rule's `check` module outside its pack gets.
fn outside(module: &str) -> String {
    format!("tasks.rules[0].check: `{module}` is outside the pack's directory")
}

#[test]
fn a_pack_check_module_outside_the_packs_directory_is_refused() {
    let rule = CHECK_RULE.replace("\"check.typ\"", "\"../check.typ\"");
    in_each_pack(&rule, "check.typ", check_x, |files, config, entry| {
        let mut files = files.to_vec();
        let above = format!("{}/../check.typ", files[0].0.trim_end_matches("/pack.toml"));
        let text = check_x(entry);
        files.push((&above, &text));

        let (_tmp, loaded) = load(&files, config);

        let err = loaded.unwrap_err().to_string();
        assert!(err.contains(&outside("../check.typ")), "{entry}: {err}");
    });
}

#[test]
fn a_pack_check_module_given_as_an_absolute_path_is_refused() {
    let tmp = crate::test_support::tempdir();
    let module = tmp.path().join("check.typ");
    std::fs::write(&module, check_x("elsewhere")).unwrap();
    let module = module.display().to_string().replace('\\', "/");
    let rule = CHECK_RULE.replace("\"check.typ\"", &format!("\"{module}\""));
    let files = [("project/team/pack.toml", rule.as_str())];

    let (_tmp, loaded) = load(&files, "extends = [\"./team\"]\n");

    let err = loaded.unwrap_err().to_string();
    assert!(err.contains(&outside(&module)), "{err}");
}

/// A layer's own rule has no pack bound, so `../shared/check.typ` reaches
/// Typst, whose world refuses a path out of the project root, as before.
#[test]
fn a_layers_own_check_module_above_the_project_meets_typsts_refusal() {
    let rule = CHECK_RULE.replace("\"check.typ\"", "\"../shared/check.typ\"");
    let shared = check_x("the shared file");
    let files = [("shared/check.typ", shared.as_str())];

    let (_tmp, loaded) = load(&files, &rule);

    let err = loaded.unwrap_err().to_string();
    assert!(!err.contains("outside the pack"), "{err}");
    assert!(err.contains("would escape the project root"), "{err}");
}
