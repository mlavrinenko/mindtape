#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "swhid-mint: host the is-this-a-SWHID-URL predicate",
  status: proposed(2026, 6, 17),
  priority: "low",
  estimate-hours: 2,
  relates-to: (
    link("swhid-mint-crate.typ"),
    link("converge-speconaut.typ"),
  ),
  tags: ("substrate",),
)

== Summary

Two repos encode "is this URL a SWHID tombstone" as a `starts_with("swh:")`
magic string: `mindtape::is_history_pointer` (`crates/mindtape/src/model.rs`) and
speconaut's `SWH_TOMBSTONE_PREFIX` + `is_ref_url` (`src/engine/dsl_links.rs`).
Co-locate the predicate with the parser that owns the scheme.

== Scope

- Add `swhid_mint::is_swhid_url(&str) -> bool` next to `Swhid::parse` (a cheap
  prefix/shape check; it need not fully parse).
- `mindtape::is_history_pointer` delegates to it.
- speconaut's prefix check delegates to it under its own task. speconaut's
  `scenario-ref` / `rule-ref` schemes stay speconaut-local.

== Acceptance

`swhid_mint::is_swhid_url` exists and is unit-tested (positive: a real
`swh:1:rev:…` tombstone; negative: `file.typ`, an http URL); `is_history_pointer`
delegates; `just check` passes.
