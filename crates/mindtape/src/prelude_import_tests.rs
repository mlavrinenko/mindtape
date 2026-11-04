#![allow(clippy::unwrap_used)]

use super::{
    PreludeImport, PreludeImportAction, PreludeRefusal, apply_rewrites, prelude_imports,
    project_prelude_pin_version,
};
use crate::config::{Config, LocalPackage, TypstConfig};

fn only(source: &str, pinned: Option<&str>) -> PreludeImport {
    let mut imports = prelude_imports(source, pinned);
    assert_eq!(imports.len(), 1, "{imports:?}");
    imports.remove(0)
}

#[test]
fn rewrites_only_the_version_of_an_unsupported_import() {
    let source = "#import \"@local/mindtape:0.1.0\": *\n#import \"../lib.typ\": *\n";
    let import = only(source, None);
    assert_eq!(import.version.as_deref(), Some("0.1.0"));
    assert_eq!(import.action, PreludeImportAction::Rewrite);
    assert_eq!(
        apply_rewrites(source, &[import]).as_deref(),
        Some("#import \"@local/mindtape:0.3.0\": *\n#import \"../lib.typ\": *\n")
    );
}

#[test]
fn keeps_a_supported_older_pin() {
    let import = only("#import \"@local/mindtape:0.2.0\": *\n", None);
    assert_eq!(import.action, PreludeImportAction::Keep);
    assert!(import.range.is_none());
}

#[test]
fn says_nothing_about_the_current_version() {
    let import = only("#import \"@local/mindtape:0.3.0\": *\n", None);
    assert_eq!(import.action, PreludeImportAction::Current);
}

#[test]
fn refuses_a_newer_version() {
    let import = only("#import \"@local/mindtape:99.99.99\": *\n", None);
    assert_eq!(
        import.action,
        PreludeImportAction::Refuse(PreludeRefusal::Newer)
    );
    assert!(import.range.is_none());
}

#[test]
fn refuses_a_malformed_spec() {
    for spec in ["@local/mindtape", "@local/mindtape:", "@local/mindtape:0.1"] {
        let source = format!("#import \"{spec}\": *\n");
        let import = only(&source, None);
        assert_eq!(
            import.action,
            PreludeImportAction::Refuse(PreludeRefusal::Malformed),
            "{spec}"
        );
    }
}

#[test]
fn refuses_the_version_a_project_pin_declares() {
    let import = only("#import \"@local/mindtape:0.1.0\": *\n", Some("0.1.0"));
    assert_eq!(
        import.action,
        PreludeImportAction::Refuse(PreludeRefusal::Pinned)
    );
    assert!(import.range.is_none());
}

#[test]
fn a_pin_for_another_version_does_not_block_the_rewrite() {
    let import = only("#import \"@local/mindtape:0.1.0\": *\n", Some("0.2.0"));
    assert_eq!(import.action, PreludeImportAction::Rewrite);
}

#[test]
fn ignores_include_comments_and_other_packages() {
    let source = "\
#include \"@local/mindtape:0.1.0\"
// #import \"@local/mindtape:0.1.0\": *
#import \"@preview/mindtape:0.1.0\": *
#import \"@local/other:0.1.0\": *
#import \"@local/mindtape-extra:0.1.0\": *
#import \"../lib.typ\": *
";
    assert!(prelude_imports(source, None).is_empty());
}

#[test]
fn ignores_a_computed_import_source() {
    let source = "\
#let version = \"0.1.0\"
#import \"@local/mindtape:\" + version: *
";
    assert!(prelude_imports(source, None).is_empty());
}

#[test]
fn preserves_every_byte_outside_the_version() {
    let source = "\
// keep this comment exactly\r
#import  \"@local/mindtape:0.1.0\"  : *\r
#let x = \"0.1.0\"\r
";
    let import = only(source, None);
    let out = apply_rewrites(source, &[import]).unwrap();
    assert_eq!(
        out,
        "\
// keep this comment exactly\r
#import  \"@local/mindtape:0.3.0\"  : *\r
#let x = \"0.1.0\"\r
"
    );
}

#[test]
fn a_multibyte_prefix_keeps_the_version_range_exact() {
    let source = "// задача\n#import \"@local/mindtape:0.1.0\": *\n";
    let import = only(source, None);
    let range = import.range.clone().unwrap();
    assert_eq!(&source[range], "0.1.0");
}

#[test]
fn reads_the_version_a_project_pin_declares() {
    let dir = crate::test_support::tempdir();
    let lib = dir.path().join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(
        lib.join("typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    let config = Config {
        typst: TypstConfig {
            local_packages: vec![LocalPackage {
                name: "mindtape".to_owned(),
                path: "lib".to_owned(),
            }],
        },
        ..Config::default()
    };
    assert_eq!(
        project_prelude_pin_version(dir.path(), &config).as_deref(),
        Some("0.1.0")
    );
}
