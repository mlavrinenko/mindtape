#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "`mt add` initial status: derive from the merged legend, not the literal `\"proposed\"`",
  status: done(2026, 7, 4),
  priority: ice(impact: 5, confidence: 0.8, ease: 7),
  tags: ("cli",),
  depends-on: (link("config-embedded-defaults.typ"),),
)

== Summary

`mt add` stamps the add-owned `status` slot with a hardcoded literal `"proposed"`
when it cannot find a non-terminal legend value
(`crates/mindtape-cli/src/add.rs:177-198`). A project whose legend has no
`"proposed"` value gets a status written that fails `check_status`
(`crates/mindtape/src/validate.rs:146`) the instant the task is validated. The
initial status must come from the project's own legend.

== Scope

- Replace the `"proposed"` fallback with: the first NON-TERMINAL entry of the
  effective (merged) legend (`Legend::entries` in lifecycle order, first with
  `terminal == false`). This is well-defined now that the foundation task
  guarantees the legend is non-empty via the floor.
- Only when the legend is genuinely empty (a project that explicitly cleared it
  with `status = []`) fall back — and then to the FIRST legend value if any, else
  leave the status slot unfilled and let validation report the real problem,
  rather than inventing `"proposed"`.
- Add a `mindtape`-side helper (e.g. `Legend::initial()` on `config.rs:275`
  returning `Option<&LegendEntry>`) so the rule is testable in the library and
  reusable by consumers, and `add.rs` calls it. Keep `add.rs` thin.

== Tests

- Legend `[a(non-terminal), b, done(terminal)]` → initial status `a`.
- Legend with a terminal FIRST entry skips it to the first non-terminal.
- Empty legend → no invented `"proposed"`; helper returns `None`.
- End-to-end: `mt add` into a project with a custom legend writes a status that
  passes `mt check`.

== Notes

- Pure follow-on cleanup; independent of the discovery/install tasks but needs the
  foundation task so "the legend is normally non-empty" holds.
- Watch the date stamp: the initial status is dated today
  (`today_ymd`) — unchanged, only the value derivation changes.
