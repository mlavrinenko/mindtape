#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Pinnable clock for reproducible output",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  tags: (),
  status: proposed(2026, 7, 11),
)

== Summary

`mt` reads the system clock directly (`crates/mindtape/src/date.rs`,
`today_ymd` via `SystemTime::now`), so `now()`, ready/blocked, `stale`, overdue,
and status stamping are all wall-clock. Any output captured from the binary
therefore changes day to day. Reproducible docs (generate-docs-cli-reference)
and deterministic tests need a pinnable "today". Add an env-driven override.

== Behaviour

- Resolve "today" once per process, in precedence order: `MINDTAPE_NOW`, then
  `SOURCE_DATE_EPOCH`, then the system clock (UTC).
- `MINDTAPE_NOW` accepts `YYYY-MM-DD` or an RFC3339 timestamp; the UTC date
  component is used, any time component parsed and dropped. The domain is
  date-only (UTC, day-granular) — nothing sub-day exists — so time is
  accepted-and-truncated for forward-compat, never rejected.
- `SOURCE_DATE_EPOCH` is epoch seconds → UTC date (the reproducible-builds
  standard).
- A malformed `MINDTAPE_NOW` / `SOURCE_DATE_EPOCH` is a hard error naming the
  offending variable and value; never silently fall through to the system clock.

== Surface

- `date.rs`: `today_iso`, `today_ymd`, `days_ago_iso` all route through the one
  resolved value — no scattered `SystemTime::now()` reads (there are two today;
  funnel both).
- The today-stamping paths (`flip`, `add`, `review`, `mt recur` advance) use the
  same resolved value, so `mt done` / `mt add` dates are stable under a pin.

== Docs

- Document `MINDTAPE_NOW` on the config and agents pages, beside
  `MINDTAPE_WORKTREE`.

== Scope

- env resolution (`MINDTAPE_NOW` / `SOURCE_DATE_EPOCH`) with precedence + errors.
- route `today_ymd` / `today_iso` / `days_ago_iso` and the stamping paths
  through the single resolved value.
- doc the env var on agents + config pages.
- regression tests: pinned date drives `now()` / `stale` / ready; RFC3339
  truncation; `SOURCE_DATE_EPOCH`; malformed → error; unset → system clock.
