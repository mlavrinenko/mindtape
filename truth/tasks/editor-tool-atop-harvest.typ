#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "typst-edit: parse-plane in-place Typst source editor",
  status: proposed(2026, 6, 16),
  priority: "high",
  estimate-hours: 10,
  relates-to: (
    link("extract-typst-harvest.typ"),
    link("mindtape-cleanup.typ"),
    link("swhid-mint-crate.typ"),
  ),
  tags: ("substrate", "cli"),
)

== Summary

Now needed (was "only if needed"): `mindtape cleanup` must rewrite links in
place, so an edit-capable layer is required. Build it as `typst-edit`, a
consumer-blind crate sibling to `typst-cst` and `typst-harvest`. Correction to
the original framing: editing is a parse-plane concern (byte spans over
`typst-syntax`), not an eval concern — it sits on the `typst-cst` plane, not atop
`typst-harvest`. Harvest says which task is terminal and what its links resolve
to; the editor says where in bytes to splice.

== Scope

- Crate `crates/typst-edit`, consumer-blind: no mention of tasks, MindTape, or
  speconaut in its docs or API; its crate docs advertise it as a generic Typst
  source editor.
- Query (over `typst-syntax` `LinkedNode`, mirroring `typst-cst`): locate
  `link("…")` targets, named-argument value spans, positional-argument spans,
  string literals — each as a byte range.
- Changeset: `Edit { range, replacement }`, validated non-overlapping and
  in-bounds, applied in one reverse-order splice. Untouched bytes preserved
  verbatim.
- Atomic by construction: a caller plans all edits across all files (parse each;
  fail loud if any is wrong-shaped) before applying any. A parse failure aborts
  before a byte is written.

== Replaces (speconaut shrink target)

speconaut's hand-rolled byte scanners — `typ_source.rs` raw-mask/slot-span,
`specs_stage_rewrite`, `cleanup_refs` `rewrite_text`, `link_scanner`,
`git_ref_scanner` — collapse onto this crate's AST-span queries plus changeset.
Parsing skips raw blocks and escaped `\#link` for free, so the raw-mask machinery
is dropped. See the speconaut task `adopt-shared-cleanup-substrate`.

== Acceptance

`crates/typst-edit` builds standalone with no task or domain knowledge; `just
check` passes; tests cover non-overlapping multi-edit apply, overlap and
out-of-bounds rejection, char-boundary safety, and link-target location including
links inside raw blocks.

== Status note (2026-06-17)

The changeset core and the `link("…")` locator landed; `mindtape cleanup` ships on
them. Remaining work, now scoped against speconaut's actual scanners (validated by
reading `typ_source.rs`, `git_ref_scanner.rs`, `link_scanner.rs`,
`specs_stage_rewrite.rs`): a generic `find_calls(source, name)` returning each
call's positional + named argument value spans and its trailing `[…]` body range,
plus a `line_of(source, offset)` helper. That one query subsumes speconaut's
raw-mask byte scanning (`compute_raw_mask`, `slot_value_span`, manual paren/quote
walking are deleted, not ported — parsing skips raw blocks for free). Drive the
shape and the speconaut adoption from `var/handoff-swhid-cleanup-substrate.md`.
