#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Configurable global date display format",
  status: done(2026, 6, 27),
  priority: ice(impact: 5, confidence: 0.85, ease: 6),
  tags: ("cli",),
)

== Summary

Dates render only as raw ISO `YYYY-MM-DD`. Users want to choose how dates
display — and globally, not per-field: one format setting drives every date
column (`due` today, any future date column), never a due-specific override.

== Scope

- Add `date_format: Option<String>` to `LsConfig`
  (`crates/mindtape/src/config.rs`), default `None` meaning ISO `YYYY-MM-DD`.
  Document it as global (applies to every rendered date), not due-only.
- Add a pure formatter to `crates/mindtape/src/date.rs` (std-only, no new
  crates): `format_iso(iso: &str, fmt: &str) -> String`. Parse via the existing
  `parse_ymd`; on parse failure return the input unchanged (defensive). Token
  substitution: `YYYY` 4-digit year, `YY` 2-digit, `MMMM` full month name,
  `MMM` abbreviated month, `MM` zero-padded month, `M` bare month, `DD`
  zero-padded day, `D` bare day. Non-token characters pass through literally.
  Longest tokens match first so `MMMM` is not eaten by `MM`.
- Wire it into `crates/mindtape-cli/src/render/table.rs`: the due cell text runs
  through `format_iso` with the configured format (fall back to `"YYYY-MM-DD"`
  when `None`). Must compose cleanly with the overdue accent — format the text,
  keep the colour decision (which still compares the underlying ISO value).
- Document the key in `crates/mindtape-cli/src/default-config.toml` under `[ls]`
  with the token table and a worked example.

== Notes

- Keep the formatter in the `mindtape` lib (not the CLI): consumer-blind and
  reusable, per the speconaut-convergence directive.
- The overdue check (sibling task `ls-accent-overdue`) must compare the raw ISO
  value, not the formatted string — formatting is display-only.
- Tests in `date.rs`: default/`"YYYY-MM-DD"` reproduces ISO; a reordered format
  like `"DD.MM.YYYY"`; `MMM`/`MMMM` month names; invalid input passes through.
