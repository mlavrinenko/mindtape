//! `[dicts.<name>]` config: named string sets.
//!
//! A dictionary names a set of strings so more than one rule can share it. It
//! is a top-level section, sibling of `[views.<name>]`, because a set is not a
//! property of tasks: an allow-list, the filter language and a future consumer
//! all read the same name.
//!
//! Three sources feed one set — `values` (inline), `path` (a file relative to
//! the project root, one entry per line, `#` comments and blank lines skipped)
//! and `include` (other dicts, project or built-in, unioned in) — and any
//! combination is legal so long as at least one is present. `case` decides how
//! a lookup compares: `fold` (the default) is ASCII-case-insensitive, `exact`
//! is byte equality.
//!
//! Two dictionaries ship embedded in the binary under the reserved `mindtape-`
//! prefix ([`BUILTIN_PREFIX`]): `mindtape-verbs` and `mindtape-dev-verbs`. They
//! are always compiled, declared or not, so a project may `include` either
//! without restating it; naming a project dict `mindtape-*` is a config error,
//! so the namespace stays the tool's.
//!
//! Everything is checked once, at config load ([`compile_dicts`]), never per
//! task: the whole section either compiles or fails as one message. The result
//! rides on the config as a [`DictCache`], because the filter language's
//! `in_dict()` runs wherever a filter runs and none of those call sites has a
//! project root to re-read the section with.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Arc, LazyLock};

use serde::{Deserialize, Serialize};

use crate::config::name_cycle::find_cycle;

/// The dictionary-name prefix reserved for the sets `mt` ships. A project dict
/// carrying it is a config error.
pub const BUILTIN_PREFIX: &str = "mindtape-";

/// The dictionaries embedded in the binary, in declaration order. Each text is
/// the data file verbatim, header comments included — they parse as comments
/// under the same reader a project's `path` file goes through, so the shipped
/// files document their own provenance without a second format.
const BUILTIN_SOURCES: &[(&str, &str)] = &[
    ("mindtape-verbs", include_str!("../dicts/verbs.txt")),
    ("mindtape-dev-verbs", include_str!("../dicts/dev-verbs.txt")),
];

/// The names of the built-in dictionaries, in declaration order.
#[must_use]
pub fn builtin_dict_names() -> Vec<&'static str> {
    BUILTIN_SOURCES.iter().map(|(name, _)| *name).collect()
}

/// `[dicts.<name>]` — one named string set, as declared in config.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DictSpec {
    /// Entries written inline.
    pub values: Vec<String>,
    /// A file of entries, one per line, relative to the project root. Blank
    /// lines and lines whose first non-blank character is `#` are skipped, and
    /// each entry is trimmed. May leave the root (`../shared/verbs.txt`), the
    /// same rule `[[typst.local-packages]] path` follows.
    pub path: Option<String>,
    /// Other dictionaries — project or built-in — whose entries are unioned in.
    /// The entries arrive raw; this dict's own `case` decides how they compare.
    pub include: Vec<String>,
    /// How a lookup compares against the entries: `fold` (the default) is
    /// ASCII-case-insensitive, `exact` is byte equality.
    pub case: DictCase,
}

/// How a compiled dictionary compares a lookup against its entries.
#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum DictCase {
    /// ASCII-case-insensitive: a capitalised title word matches a lowercase
    /// entry. The default.
    #[default]
    Fold,
    /// Byte equality.
    Exact,
}

/// One compiled dictionary: the resolved entry set, already normalised for the
/// case policy, so a lookup is a set probe and nothing more.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dict {
    entries: BTreeSet<String>,
    case: DictCase,
}

impl Dict {
    /// Compile `entries` under `case`, normalising them so [`Dict::contains`]
    /// never has to fold both sides.
    fn new(entries: BTreeSet<String>, case: DictCase) -> Self {
        let entries = match case {
            DictCase::Exact => entries,
            DictCase::Fold => entries.iter().map(|e| e.to_ascii_lowercase()).collect(),
        };
        Self { entries, case }
    }

    /// Is `value` in this dictionary, under its own case policy?
    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        match self.case {
            DictCase::Exact => self.entries.contains(value),
            DictCase::Fold => self.entries.contains(&value.to_ascii_lowercase()),
        }
    }

    /// How this dictionary compares a lookup.
    #[must_use]
    pub fn case(&self) -> DictCase {
        self.case
    }

    /// How many entries it resolved to.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// A compiled dictionary is never empty — that is a config error — so this
    /// is always `false`; it exists because `len` without it is a clippy lint.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries as stored (folded to lowercase under `fold`), sorted.
    pub fn entries(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(String::as_str)
    }
}

/// Every compiled dictionary for a project: the declared `[dicts.<name>]`
/// sections plus the built-ins, which are always present.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dicts(BTreeMap<String, Dict>);

impl Dicts {
    /// The dictionary named `name`, or `None` if no such dictionary exists.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Dict> {
        self.0.get(name)
    }

    /// Is `value` in the dictionary named `name`? An unknown name reads as a
    /// miss — the caller validates the name where it is written.
    #[must_use]
    pub fn contains(&self, name: &str, value: &str) -> bool {
        self.get(name).is_some_and(|dict| dict.contains(value))
    }

    /// Every compiled dictionary name, sorted.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }
}

/// The built-in dictionaries alone, compiled once per process. Cloning a
/// [`DictCache`] is an [`Arc`] bump off this, so a `Config` carrying no
/// `[dicts]` section costs one pointer rather than every entry.
static BUILTIN_DICTS: LazyLock<Arc<Dicts>> = LazyLock::new(|| Arc::new(builtin_dicts()));

/// Compile the embedded dictionaries directly, bypassing [`compile_dicts`]:
/// they have no `path`, no `include` and no way to fail, so building them here
/// needs no `Result` for a caller to swallow.
fn builtin_dicts() -> Dicts {
    Dicts(
        BUILTIN_SOURCES
            .iter()
            .map(|(name, text)| {
                let entries = parse_entries(text).into_iter().collect();
                ((*name).to_owned(), Dict::new(entries, DictCase::Fold))
            })
            .collect(),
    )
}

/// The compiled `[dicts]` section as [`crate::config::Config`] carries it, so
/// the filter language's `in_dict()` has the entries without a project root in
/// hand.
///
/// A cache, not a declaration — which is why [`PartialEq`] here is always
/// `true`. Its contents are a function of the `[dicts]` section (which `Config`
/// compares, field by field) and of the project root a `path` entry resolves
/// against, which `Config` does not carry at all. Comparing the cache would
/// therefore make two configs with byte-identical declared settings unequal for
/// having been loaded from different directories, a distinction `Config`
/// equality has never claimed to draw.
///
/// It defaults to the built-ins alone, so `in_dict("mindtape-verbs", ...)`
/// resolves against any `Config` — including `Config::default()` and one
/// deserialized without ever touching a filesystem.
#[derive(Debug, Clone)]
pub struct DictCache(Arc<Dicts>);

impl Default for DictCache {
    fn default() -> Self {
        Self(Arc::clone(&BUILTIN_DICTS))
    }
}

impl PartialEq for DictCache {
    /// Always equal — see the type's own documentation for why a derived
    /// comparison would be the wrong answer.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl From<Dicts> for DictCache {
    fn from(dicts: Dicts) -> Self {
        Self(Arc::new(dicts))
    }
}

impl DictCache {
    /// The compiled dictionaries this cache holds.
    #[must_use]
    pub fn dicts(&self) -> &Dicts {
        &self.0
    }
}

/// Read one dictionary file's text: entries one per line, blank lines and `#`
/// comment lines skipped, each entry trimmed.
fn parse_entries(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

/// The checks that need no filesystem: the reserved prefix, a dict with no
/// source at all, an unknown `include` target (with a nearest-match
/// suggestion), and an `include` cycle. Returns one message per problem.
///
/// Run from [`crate::config::Config::validate`], so every path that validates a
/// config catches them, and again from [`compile_dicts`], which additionally
/// needs the graph proven acyclic before it walks it.
#[must_use]
pub fn validate_dicts(dicts: &BTreeMap<String, DictSpec>) -> Vec<String> {
    let mut problems = Vec::new();
    let known: Vec<String> = builtin_dict_names()
        .into_iter()
        .map(ToOwned::to_owned)
        .chain(dicts.keys().cloned())
        .collect();
    let mut refs: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (name, spec) in dicts {
        if name.starts_with(BUILTIN_PREFIX) {
            problems.push(format!(
                "dicts.{name}: the `{BUILTIN_PREFIX}` prefix is reserved for built-in dictionaries"
            ));
        }
        if spec.values.is_empty() && spec.include.is_empty() && spec.path.is_none() {
            problems.push(format!(
                "dicts.{name}: no source — set at least one of `values`, `path` or `include`"
            ));
        }
        for target in &spec.include {
            if !known.contains(target) {
                problems.push(format!(
                    "dicts.{name}: unknown include `{target}`{}",
                    crate::suggest::hint("dict", target, &known)
                ));
            }
        }
        refs.insert(name.as_str(), spec.include.clone());
    }
    if let Some(cycle) = find_cycle(&refs) {
        problems.push(format!("dicts: include cycle {}", cycle.join(" → ")));
    }
    problems
}

/// One dictionary's own entries: `values`, plus `path`'s lines when it has one
/// and there is a root to resolve it against. `path` may leave the root.
fn own_entries(name: &str, spec: &DictSpec, root: Option<&Path>) -> Result<Vec<String>, String> {
    let mut entries = spec.values.clone();
    if let (Some(rel), Some(root)) = (&spec.path, root) {
        let full = root.join(rel);
        let text = std::fs::read_to_string(&full)
            .map_err(|err| format!("dicts.{name}.path: cannot read `{}`: {err}", full.display()))?;
        entries.extend(parse_entries(&text));
    }
    Ok(entries)
}

/// Union `name`'s own entries with every dictionary it includes, transitively.
/// Safe to recurse: [`compile_dicts`] proves the graph acyclic first.
fn resolve(
    name: &str,
    own: &BTreeMap<String, Vec<String>>,
    includes: &BTreeMap<String, Vec<String>>,
    memo: &mut BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    if let Some(done) = memo.get(name) {
        return done.clone();
    }
    let mut set: BTreeSet<String> = own.get(name).into_iter().flatten().cloned().collect();
    for target in includes.get(name).into_iter().flatten() {
        set.extend(resolve(target, own, includes, memo));
    }
    memo.insert(name.to_owned(), set.clone());
    set
}

/// Compile every `[dicts.<name>]` section against `root`, plus the built-ins.
///
/// This is the whole validation seam for the section: it runs
/// [`validate_dicts`], reads each `path`, resolves the `include` graph, and
/// refuses a dictionary that ends up with no entries. Called once at config
/// load, so a broken dictionary is one config error rather than one per task.
///
/// # Errors
/// Returns every problem found, joined with `; ` — a reserved name, an unknown
/// or cyclic `include`, an unreadable `path`, a dict with no source, or one
/// that resolves empty.
pub fn compile_dicts(dicts: &BTreeMap<String, DictSpec>, root: &Path) -> Result<Dicts, String> {
    compile_dicts_in(dicts, Some(root))
}

/// [`compile_dicts`] with no project root: a `path` source contributes nothing,
/// because there is nothing to resolve it against. Every other check still
/// runs.
///
/// This is what [`crate::config::parse_config`] hydrates its [`DictCache`]
/// with — it is handed a string and never learns where the project lives — so a
/// `Config` built from text alone still answers `in_dict()` for its inline and
/// `include`-sourced dictionaries. Config load re-compiles against the real
/// root ([`crate::config::layers`]) and that result wins.
///
/// # Errors
/// As [`compile_dicts`], minus the unreadable-`path` case — a dict sourced from
/// a `path` alone resolves to no entries here and fails on that instead.
pub(crate) fn compile_dicts_rootless(dicts: &BTreeMap<String, DictSpec>) -> Result<Dicts, String> {
    compile_dicts_in(dicts, None)
}

/// The shared body of [`compile_dicts`] and [`compile_dicts_rootless`].
fn compile_dicts_in(
    dicts: &BTreeMap<String, DictSpec>,
    root: Option<&Path>,
) -> Result<Dicts, String> {
    let problems = validate_dicts(dicts);
    if !problems.is_empty() {
        return Err(problems.join("; "));
    }
    let mut own: BTreeMap<String, Vec<String>> = BUILTIN_SOURCES
        .iter()
        .map(|(name, text)| ((*name).to_owned(), parse_entries(text)))
        .collect();
    let mut includes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut problems = Vec::new();
    for (name, spec) in dicts {
        match own_entries(name, spec, root) {
            Ok(entries) => drop(own.insert(name.clone(), entries)),
            Err(msg) => problems.push(msg),
        }
        includes.insert(name.clone(), spec.include.clone());
    }
    if !problems.is_empty() {
        return Err(problems.join("; "));
    }
    let mut memo = BTreeMap::new();
    let mut compiled = BTreeMap::new();
    for (name, case) in compile_order(dicts) {
        let entries = resolve(&name, &own, &includes, &mut memo);
        if entries.is_empty() {
            problems.push(format!("dicts.{name}: resolves to no entries"));
            continue;
        }
        compiled.insert(name, Dict::new(entries, case));
    }
    if problems.is_empty() {
        Ok(Dicts(compiled))
    } else {
        Err(problems.join("; "))
    }
}

/// Every dictionary to compile and the case policy it compiles under: the
/// built-ins (always `fold` — their data files are lowercase) followed by the
/// project's own.
fn compile_order(dicts: &BTreeMap<String, DictSpec>) -> Vec<(String, DictCase)> {
    builtin_dict_names()
        .into_iter()
        .map(|name| (name.to_owned(), DictCase::Fold))
        .chain(dicts.iter().map(|(name, spec)| (name.clone(), spec.case)))
        .collect()
}

#[cfg(test)]
#[path = "dicts_tests.rs"]
mod tests;
