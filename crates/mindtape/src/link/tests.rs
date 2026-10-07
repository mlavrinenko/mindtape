#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use super::{
    LinkKind, LinksConfig, NoteRule, TargetValidator, acyclic_kinds, blocking_kinds,
    is_builtin_kind, kind_names, kind_targets_tasks, parse_links, resolve_kind,
};

/// A custom kind carrying only its name — every other field at its default
/// (unbounded, optional note, non-blocking, acyclic off, `local-task-path`).
fn custom(name: &str) -> LinkKind {
    LinkKind {
        name: name.to_owned(),
        ..LinkKind::default()
    }
}

/// A links config holding exactly `kinds`.
fn with_kinds(kinds: Vec<LinkKind>) -> LinksConfig {
    LinksConfig { kinds }
}

fn dict(pairs: Vec<(&str, HVal)>) -> HVal {
    HVal::Dict(
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn edge(task: &str, kind: &str, note: Option<&str>) -> HVal {
    dict(vec![
        ("task", HVal::Str(task.to_owned())),
        ("kind", HVal::Str(kind.to_owned())),
        ("note", note.map_or(HVal::Null, |n| HVal::Str(n.to_owned()))),
    ])
}

/// [`parse_links`] without caring about shape errors — most tests here feed
/// it a well-shaped array; shape-error tests call `parse_links` directly.
fn parse(value: Option<&HVal>) -> Vec<super::Link> {
    parse_links(value, &mut Vec::new())
}

#[test]
fn parse_links_absent_is_empty() {
    assert_eq!(parse(None), Vec::new());
}

#[test]
fn parse_links_reads_dict_edges() {
    let arr = HVal::Array(vec![
        edge("a.typ", "depends-on", None),
        edge("b.typ", "related", Some("why")),
    ]);
    let links = parse(Some(&arr));
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].task, "a.typ");
    assert_eq!(links[0].kind, "depends-on");
    assert_eq!(links[0].note, None);
    assert_eq!(links[1].task, "b.typ");
    assert_eq!(links[1].kind, "related");
    assert_eq!(links[1].note.as_deref(), Some("why"));
}

#[test]
fn parse_links_bare_string_gets_empty_kind() {
    // No helper used: kept (not silently dropped) so validation can flag it as
    // an unknown-kind edge — never a matching built-in or config kind.
    let arr = HVal::Array(vec![HVal::Str("bare.typ".to_owned())]);
    let links = parse(Some(&arr));
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].task, "bare.typ");
    assert_eq!(links[0].kind, "");
}

#[test]
fn parse_links_non_array_reports_shape_error_not_silent_empty() {
    let mut errors = Vec::new();
    let value = HVal::Str("a.typ".to_owned());
    let links = parse_links(Some(&value), &mut errors);
    assert!(links.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "links");
    assert_eq!(errors[0].message, "expected an array, found a string");
}

#[test]
fn parse_links_flattens_the_array_spelling_of_concatenation() {
    // `links: (depends-on("a.typ"), depends-on("b.typ"))` — each helper returns
    // an array, so the array spelling nests. Every edge survives; the `+` form
    // and this one differ only in source text.
    let arr = HVal::Array(vec![
        HVal::Array(vec![edge("a.typ", "depends-on", None)]),
        HVal::Array(vec![edge("b.typ", "depends-on", None)]),
    ]);
    let mut errors = Vec::new();
    let links = parse_links(Some(&arr), &mut errors);
    assert!(
        errors.is_empty(),
        "the array form is a spelling, not an error"
    );
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].task, "a.typ");
    assert_eq!(links[1].task, "b.typ");
}

#[test]
fn parse_links_keeps_a_nested_entry_mixed_with_a_flat_one() {
    // The partial-loss case: `depends-on("a.typ") + (depends-on("b.typ"),)`
    // used to keep exactly one of two edges — half a dependency graph, which is
    // harder to notice than none of it.
    let arr = HVal::Array(vec![
        edge("a.typ", "depends-on", None),
        HVal::Array(vec![HVal::Array(vec![edge(
            "b.typ",
            "related",
            Some("why"),
        )])]),
    ]);
    let mut errors = Vec::new();
    let links = parse_links(Some(&arr), &mut errors);
    assert!(errors.is_empty());
    assert_eq!(links.len(), 2);
    assert_eq!(links[1].task, "b.typ");
    assert_eq!(links[1].kind, "related");
    assert_eq!(links[1].note.as_deref(), Some("why"));
}

#[test]
fn parse_links_reports_a_non_edge_entry_by_position() {
    // Neither an edge nor a nest: the drop that flattening must not re-hide.
    let arr = HVal::Array(vec![
        edge("a.typ", "depends-on", None),
        HVal::Array(vec![HVal::Int(42)]),
    ]);
    let mut errors = Vec::new();
    let links = parse_links(Some(&arr), &mut errors);
    assert_eq!(links.len(), 1);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "links[1][0]");
    assert_eq!(
        errors[0].message,
        "expected an edge like depends-on(\"other.typ\"), found a number"
    );
}

#[test]
fn parse_links_empty_nest_contributes_nothing_and_reports_nothing() {
    let arr = HVal::Array(vec![HVal::Array(Vec::new())]);
    let mut errors = Vec::new();
    assert!(parse_links(Some(&arr), &mut errors).is_empty());
    assert!(errors.is_empty());
}

#[test]
fn parse_links_excludes_history_pointers() {
    let arr = HVal::Array(vec![
        edge(
            "swh:1:rev:0123456789abcdef0123456789abcdef01234567;path=tasks/gone.typ",
            "depends-on",
            None,
        ),
        edge("live.typ", "depends-on", None),
    ]);
    let links = parse(Some(&arr));
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].task, "live.typ");
}

#[test]
fn kind_names_lists_builtins_then_each_custom_name_once() {
    // A custom entry named after a built-in, and a duplicate custom name, must
    // not add a second copy: the enumeration is the candidate set a writer
    // suggests, and a repeated kind there is noise.
    let config = with_kinds(vec![custom("depends-on"), custom("gates"), custom("gates")]);
    assert_eq!(
        kind_names(&config),
        vec![
            "depends-on",
            "parent",
            "recurred-from",
            "supersedes",
            "duplicates",
            "related",
            "gates",
        ]
    );
}

#[test]
fn resolve_kind_finds_every_builtin() {
    let config = LinksConfig::default();
    for name in [
        "depends-on",
        "parent",
        "recurred-from",
        "supersedes",
        "duplicates",
        "related",
    ] {
        assert!(
            resolve_kind(name, &config).is_some(),
            "{name} should resolve"
        );
        assert!(is_builtin_kind(name));
    }
}

#[test]
fn resolve_kind_unknown_is_none() {
    let config = LinksConfig::default();
    assert!(resolve_kind("bogus", &config).is_none());
    assert!(!is_builtin_kind("bogus"));
}

#[test]
fn resolve_kind_finds_custom_config_kind() {
    let config = with_kinds(vec![custom("blocks-release")]);
    let rule = resolve_kind("blocks-release", &config).expect("custom kind resolves");
    assert_eq!(rule.max, None);
    assert!(!is_builtin_kind("blocks-release"));
}

#[test]
fn builtin_cannot_be_shadowed_by_config() {
    // A config entry named after a built-in never wins: the built-in's rule
    // (cardinality 1 for `parent`) always resolves.
    let config = with_kinds(vec![custom("parent")]);
    let rule = resolve_kind("parent", &config).unwrap();
    assert_eq!(rule.max, Some(1));
}

#[test]
fn blocking_kinds_default_is_depends_on_only() {
    let config = LinksConfig::default();
    assert_eq!(blocking_kinds(&config), vec!["depends-on"]);
}

#[test]
fn acyclic_kinds_default_is_depends_on_and_parent() {
    let config = LinksConfig::default();
    let kinds = acyclic_kinds(&config);
    assert!(kinds.contains(&"depends-on"));
    assert!(kinds.contains(&"parent"));
    assert_eq!(kinds.len(), 2);
}

#[test]
fn custom_kind_can_opt_into_blocking_and_acyclic() {
    let config = with_kinds(vec![LinkKind {
        blocking: true,
        acyclic: true,
        ..custom("gates")
    }]);
    assert!(blocking_kinds(&config).contains(&"gates"));
    assert!(acyclic_kinds(&config).contains(&"gates"));
}

#[test]
fn custom_kind_not_blocking_excluded_from_blocking_kinds() {
    let config = with_kinds(vec![custom("gates")]);
    assert_eq!(blocking_kinds(&config), vec!["depends-on"]);
}

#[test]
fn custom_kind_not_acyclic_excluded_from_acyclic_kinds() {
    let config = with_kinds(vec![custom("gates")]);
    assert_eq!(acyclic_kinds(&config), vec!["depends-on", "parent"]);
}

/// A kind pointing outside the backlog takes no part in the task graph, even
/// when it opts into `blocking`/`acyclic`: those flags describe task↔task
/// edges, and this kind has none.
#[test]
fn external_kind_never_blocks_or_gates_cycles() {
    let config = with_kinds(vec![LinkKind {
        blocking: true,
        acyclic: true,
        validate: TargetValidator::Url,
        ..custom("discharged-by")
    }]);
    assert_eq!(blocking_kinds(&config), vec!["depends-on"]);
    assert_eq!(acyclic_kinds(&config), vec!["depends-on", "parent"]);
}

#[test]
fn kind_targets_tasks_reads_the_validate_axis() {
    let config = with_kinds(vec![
        custom("local"),
        LinkKind {
            validate: TargetValidator::GitSha,
            ..custom("commit")
        },
    ]);
    // Built-ins are fixed at `local-task-path`; an unknown kind is treated as
    // one so it still reports unresolved rather than skipping resolution.
    assert!(kind_targets_tasks("depends-on", &config));
    assert!(kind_targets_tasks("bogus", &config));
    assert!(kind_targets_tasks("local", &config));
    assert!(!kind_targets_tasks("commit", &config));
}

/// The default `validate` is `local-task-path`, so a config written before the
/// field existed keeps every kind in the task graph.
#[test]
fn custom_kind_defaults_to_local_task_path() {
    assert_eq!(
        LinkKind::default().validate,
        TargetValidator::LocalTaskPath,
        "an omitted `validate` must keep today's behaviour"
    );
    assert!(LinkKind::default().validate.targets_tasks());
    assert_eq!(LinkKind::default().note, NoteRule::Optional);
}
