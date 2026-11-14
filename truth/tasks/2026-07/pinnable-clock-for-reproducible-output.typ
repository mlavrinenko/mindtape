#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Pinnable clock for reproducible output",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  tags: (),
  status: done(2026, 7, 12),
)

== Summary

`mt` reads the system clock directly (`crates/mindtape/src/date.rs`,
`today_ymd` via `SystemTime::now`), so `now()`, ready/blocked, `stale`, overdue,
and status stamping are all wall-clock. Any output captured from the binary
changes from day to day. Reproducible docs (generate-docs-cli-reference) and
deterministic tests need a fixed "today". Add an env-driven override.

== Behaviour

- Resolve "today" once per process from `MINDTAPE_NOW`, else the system clock
  (UTC).
- `MINDTAPE_NOW` accepts `YYYY-MM-DD` or an RFC3339 timestamp. The parser uses
  the UTC date component and drops any time component. The domain is date-only
  (UTC, by day), and nothing sub-day exists. The parser accepts and truncates time
  for forward compatibility, never rejected.
- A malformed `MINDTAPE_NOW` raises a hard error that reports the variable and
  value. Never fall through to the system clock silently.
- `SOURCE_DATE_EPOCH` stays deliberately unhonoured. For `mt`, "today" is an
  operational query concept, not a build-artifact timestamp. Nix dev and CI
  shells export `SOURCE_DATE_EPOCH=315532800` (1980-01-01). Honouring it would
  time-travel every `mt ls` there. Reproducible output pins `MINDTAPE_NOW`.

== Surface

- `date.rs`: `today_iso`, `today_ymd`, `days_ago_iso` all route through the one
  resolved value, with no scattered `SystemTime::now()` reads. Two such reads
  exist today, and both funnel through it.
- The today-stamping paths (`flip`, `add`, `review`, `mt recur` advance) use the
  same resolved value, so `mt done` / `mt add` dates are stable under a pin.

== Docs

- Document `MINDTAPE_NOW` on the config and agents pages, beside
  `MINDTAPE_WORKTREE`.

== Scope

- environment resolution (`MINDTAPE_NOW`) with errors.
- route `today_ymd` / `today_iso` / `days_ago_iso` and the stamping paths
  through the single resolved value.
- document the environment variable on the agents and config pages.
- Regression tests: a pinned date drives `now()`, `stale` and ready. RFC3339
  truncation. A malformed value errors. An unset value falls back to the system
  clock.
