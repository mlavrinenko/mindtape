#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "State the recur gap in the canonical module doc",
  kind: "docs",
  status: done(
    2026,
    9,
    19,
  )[the module doc names the recur gap and points at the schema page],
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("docs", "tooling"),
  estimate: (files: 2),
  assignee: "agent",
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[a Rust reader stops believing `recur` survives a JSON round-trip],
      ),
      "readable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[the module doc agrees with `CanonicalTask` and with the schema page],
      ),
      "sound-codebase",
    ),
  ),
)

== AS-IS

1. 🔴 `crates/mindtape/src/canonical.rs`'s module doc says `mt ls --json | mt
  import` round-trips, but `CanonicalTask` has no `recur` and `mt import`
  writes none. `www/content/schema.typ` now names that gap, so the Rust doc is
  the one place still claiming otherwise.

== TO-BE

- 🟢 The module doc says every field round-trips except `recur`, points at the
  schema page as the user-facing account, and drops the stale "in a later
  task" history.

== Source of truth

- `www/content/schema.typ` — the page that names the gap for users.
- `crates/mindtape/src/import_map.rs` — where `recur` is hardcoded to `none`.

== Changes

- `crates/mindtape/src/canonical.rs` — the module doc's opening paragraph.

== Out of scope

- Any behavior change to `mt import` or the schema itself.

== Dogfood

The module doc is read on every visit to the type; `just check` covers the
comment-free build, and the docs gate keeps the page it points at honest.
