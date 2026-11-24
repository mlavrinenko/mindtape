//! Shared fixtures for the `add` tests: the template sources, the seed maps,
//! the scaffold and parse helpers, and the values-file and editor-script
//! writers. Each helper replaces a setup the test files repeated.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;
use std::path::Path;

use mindtape::{AddConfig, FieldOp, LinkValue, RawValue, SlotSpec, discover_slots};

use super::interactive::parse_buffer;
use super::scaffold::scaffold;
use super::values::{Collected, collect_values};

/// A template with one required `title` slot.
pub(crate) const TITLE_SOURCE: &str = r#"#show: task.with(title: slot("title"))"#;

/// A template with one required `x` slot.
pub(crate) const X_SOURCE: &str = r#"#show: task.with(x: slot("x"))"#;

/// A template with one optional `x` slot.
pub(crate) const X_OPTIONAL_SOURCE: &str = r#"#show: task.with(x: slot("x", required: false))"#;

/// A template with one `text` slot.
pub(crate) const X_TEXT_SOURCE: &str = r#"#show: task.with(x: slot("x", kind: "text"))"#;

/// A template with one `float` slot.
pub(crate) const X_FLOAT_SOURCE: &str = r#"#show: task.with(x: slot("x", kind: "float"))"#;

/// A template with one `bool` slot.
pub(crate) const X_BOOL_SOURCE: &str = r#"#show: task.with(x: slot("x", kind: "bool"))"#;

/// A template with one `links` slot.
pub(crate) const LINKS_SOURCE: &str =
    r#"#show: task.with(links: slot("links", kind: "links", default: ()))"#;

/// A template with one `tags` slot whose default is a non-empty list.
pub(crate) const TAGS_SOURCE: &str =
    r#"#show: task.with(tags: slot("tags", kind: "list", default: ("backend",)))"#;

/// A template with one `goals` slot.
pub(crate) const GOALS_SOURCE: &str = r#"#show: task.with(goals: slot("goals", kind: "raw"))"#;

/// The slots `source` declares.
pub(crate) fn specs_of(source: &str) -> Vec<SlotSpec> {
    discover_slots(source)
}

/// The scaffold `source` renders with no seed values.
pub(crate) fn scaffold_of(source: &str) -> String {
    scaffold(
        &AddConfig::default(),
        &discover_slots(source),
        &BTreeMap::new(),
    )
}

/// The scaffold `source` renders with `seed`.
pub(crate) fn scaffold_with(source: &str, seed: &BTreeMap<String, RawValue>) -> String {
    scaffold(&AddConfig::default(), &discover_slots(source), seed)
}

/// The scaffold `source` renders with `config` and no seed values.
pub(crate) fn scaffold_with_config(config: &AddConfig, source: &str) -> String {
    scaffold(config, &discover_slots(source), &BTreeMap::new())
}

/// An [`AddConfig`] whose header is `text`.
pub(crate) fn header_config(text: &str) -> AddConfig {
    AddConfig {
        header: Some(text.to_owned()),
        ..Default::default()
    }
}

/// An [`AddConfig`] with comments off.
pub(crate) fn comments_off() -> AddConfig {
    AddConfig {
        comments: false,
        ..Default::default()
    }
}

/// An [`AddConfig`] that always opens the editor.
pub(crate) fn always_config() -> AddConfig {
    AddConfig {
        interactive: mindtape::InteractiveMode::Always,
        ..Default::default()
    }
}

/// An [`AddConfig`] that never opens the editor.
pub(crate) fn never_config() -> AddConfig {
    AddConfig {
        interactive: mindtape::InteractiveMode::Never,
        ..Default::default()
    }
}

/// A seed map from `pairs`.
pub(crate) fn seed(pairs: &[(&str, RawValue)]) -> BTreeMap<String, RawValue> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

/// A list of strings.
pub(crate) fn list(items: &[&str]) -> RawValue {
    RawValue::List(items.iter().map(|item| (*item).to_owned()).collect())
}

/// The two links the link fixtures share.
pub(crate) fn links() -> RawValue {
    RawValue::Links(vec![
        LinkValue {
            task: "origin.typ".to_owned(),
            kind: "surfaced-by".to_owned(),
            note: None,
        },
        LinkValue {
            task: "other.typ".to_owned(),
            kind: "related".to_owned(),
            note: Some("why".to_owned()),
        },
    ])
}

/// The buffer `text` parses to.
pub(crate) fn parse(text: &str) -> BTreeMap<String, RawValue> {
    parse_buffer(text).unwrap()
}

/// Write `text` to `dir`/`name` and return the path as a string.
pub(crate) fn values_file(dir: &Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

/// A file in `dir` named `name` holding `body`, as a path string.
pub(crate) fn file(dir: &tempfile::TempDir, name: &str, body: &str) -> String {
    values_file(dir.path(), name, body)
}

/// A temp dir and a file in it named `name` holding `body`.
pub(crate) fn file_fixture(name: &str, body: &str) -> (tempfile::TempDir, String) {
    let dir = mindtape::test_support::tempdir();
    let path = file(&dir, name, body);
    (dir, path)
}

/// Write `text` to `dir`/`v.json` and return the path as a string.
pub(crate) fn json_file(dir: &Path, text: &str) -> String {
    values_file(dir, "v.json", text)
}

/// Write the shared `title`/`impact` YAML fixture to `dir`/`v.yaml` and return
/// the path as a string.
pub(crate) fn yaml_values_file(dir: &Path) -> String {
    values_file(dir, "v.yaml", "title: From YAML\nimpact: 8\n")
}

/// Write the shared JSON values fixture to `dir`/`v.json` and return the path.
pub(crate) fn json_values_file(dir: &Path) -> String {
    json_file(dir, r#"{"title":"From JSON","impact":8,"flag":true}"#)
}

/// Write the shared `.yml` values fixture to `dir`/`v.yml` and return the path.
pub(crate) fn yml_values_file(dir: &Path) -> String {
    values_file(dir, "v.yml", "title: From Yml\n")
}

/// Write the shared unterminated YAML fixture to `dir`/`v.yaml` and return the
/// path.
pub(crate) fn unterminated_yaml_file(dir: &Path) -> String {
    values_file(dir, "v.yaml", "title: [unterminated\n")
}

/// The final map `collect_values` builds from `paths`, `sets` and `tokens`.
pub(crate) fn collect_map(
    paths: &[String],
    sets: &[String],
    tokens: &[String],
) -> BTreeMap<String, RawValue> {
    let collected = collect_values(paths, sets, tokens).unwrap();
    let mut out = BTreeMap::new();
    for (name, op) in &collected.ops {
        match op {
            FieldOp::Set(value) => {
                out.insert(name.clone(), value.clone());
            }
            FieldOp::Append(item) => {
                super::tokens::append_list_value(&mut out, name, item.clone());
            }
        }
    }
    out
}

/// The error `collect_values` reports for `paths`, `sets` and `tokens`.
pub(crate) fn collect_err(paths: &[String], sets: &[String], tokens: &[String]) -> String {
    collect_values(paths, sets, tokens).unwrap_err().to_string()
}

/// The error `collect_values` reports for `path`.
pub(crate) fn collect_err_at(path: &str) -> String {
    collect_err(&[path.to_owned()], &[], &[])
}

/// The values `collect_map` builds from `path` and `title=Override`.
pub(crate) fn collect_override(path: &str) -> BTreeMap<String, RawValue> {
    collect_map(&[path.to_owned()], &["title=Override".to_owned()], &[])
}

/// The values `collect_map` builds from `path`.
pub(crate) fn collect_path(path: &str) -> BTreeMap<String, RawValue> {
    collect_map(&[path.to_owned()], &[], &[])
}

/// The values `apply_token` builds from `summary@path`.
pub(crate) fn applied_file(path: &str) -> BTreeMap<String, RawValue> {
    let mut out = BTreeMap::new();
    super::tokens::apply_token(&mut out, &format!("summary@{path}")).unwrap();
    out
}

/// The operations `sets` collect.
pub(crate) fn collected(sets: &[&str]) -> Collected {
    collect_values(&[], &owned(sets), &[]).unwrap()
}

/// The slot values `sets` produce against `source`.
pub(crate) fn slot_values(source: &str, sets: &[&str]) -> BTreeMap<String, RawValue> {
    collected(sets)
        .slot_values(&discover_slots(source))
        .unwrap()
}

/// `items` as owned strings.
pub(crate) fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

/// `pairs` as `Set` operations.
pub(crate) fn ops(pairs: &[(&str, &str)]) -> Vec<(String, FieldOp)> {
    pairs
        .iter()
        .map(|(name, value)| {
            (
                (*name).to_owned(),
                FieldOp::Set(RawValue::Str((*value).to_owned())),
            )
        })
        .collect()
}

/// The refusal `reject_unknown_keys` reports for `pairs` against the built-in
/// template's slots.
pub(crate) fn reject_plain(pairs: &[(&str, &str)]) -> String {
    let specs = discover_slots(super::values::BUILTIN_TASK);
    crate::diag::plain(&super::values::reject_unknown_keys(&ops(pairs), &specs).unwrap_err())
}

/// The JSON `describe` renders for `specs`.
pub(crate) fn describe_json(specs: &[SlotSpec]) -> serde_json::Value {
    let json = super::values::describe(specs, super::values::DescribeFormat::Json).unwrap();
    serde_json::from_str(&json).unwrap()
}

/// The text `describe` renders for `specs`.
pub(crate) fn describe_text(specs: &[SlotSpec]) -> String {
    super::values::describe(specs, super::values::DescribeFormat::Text).unwrap()
}

/// A [`mindtape::Config`] whose naming dedup mode is `mode`.
pub(crate) fn config_with_dedup(mode: mindtape::model::naming::DedupMode) -> mindtape::Config {
    let mut config = mindtape::Config::default();
    config.tasks.naming.dedup = Some(mode);
    config
}

/// A [`mindtape::Config`] whose naming stem is `{slug}`.
pub(crate) fn stem_config() -> mindtape::Config {
    let mut config = mindtape::Config::default();
    config.tasks.naming.stem = Some("{slug}".to_owned());
    config
}

/// Assert `effective_mode` picks `want` for `flag` and `config`.
pub(crate) fn assert_effective_mode(
    flag: Option<super::dedup::DedupArg>,
    config: &mindtape::Config,
    want: mindtape::model::naming::DedupMode,
) {
    assert_eq!(effective_mode_of(flag, config), want);
}

/// The dedup mode `effective_mode` picks for `flag` and `config`.
pub(crate) fn effective_mode_of(
    flag: Option<super::dedup::DedupArg>,
    config: &mindtape::Config,
) -> mindtape::model::naming::DedupMode {
    super::dedup::effective_mode(flag, config)
}

/// `resolve` on the shared `foo` stem and `tasks/foo.typ` label.
fn resolve_at(
    mode: mindtape::model::naming::DedupMode,
    exists: impl Fn(&str) -> bool,
) -> Result<super::dedup::Collision, crate::diag::Diag> {
    super::dedup::resolve(mode, "foo", "tasks/foo.typ", exists)
}

/// The collision `resolve` reports for `mode` and `exists`.
pub(crate) fn resolve_ok(
    mode: mindtape::model::naming::DedupMode,
    exists: impl Fn(&str) -> bool,
) -> super::dedup::Collision {
    resolve_at(mode, exists).unwrap()
}

/// The error `resolve` reports for `mode` and `exists`.
pub(crate) fn resolve_err(
    mode: mindtape::model::naming::DedupMode,
    exists: impl Fn(&str) -> bool,
) -> crate::diag::Diag {
    resolve_at(mode, exists).unwrap_err()
}

/// An editor command that overwrites the file it receives with `line`.
#[cfg(unix)]
pub(crate) fn editor_script(dir: &Path, line: &str) -> String {
    let script = dir.join("editor.sh");
    std::fs::write(&script, format!("cat > \"$1\" << 'EOF'\n{line}\nEOF\n")).unwrap();
    format!("sh {}", script.display())
}

/// An editor command that overwrites the file it receives with `line`. Not
/// `bash`: on Windows that can resolve to the WSL launcher in `System32`.
#[cfg(windows)]
pub(crate) fn editor_script(dir: &Path, line: &str) -> String {
    let script = dir.join("editor.cmd");
    std::fs::write(&script, format!("@echo {line}> %1\r\n")).unwrap();
    format!("cmd /C {}", script.display())
}
