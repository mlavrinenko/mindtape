#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Configurable default list sort",
  status: done(2026, 6, 20),
  priority: ice(impact: 5, confidence: 0.6, ease: 7),
  estimate: (hours: 3),
  relates-to: (link("ready-state-filter-predicates.typ"),),
  tags: ("cli",),
)

== Summary

Let a project pick the order `mindtape list` uses when the caller passes no
`--sort`. Today that order is hard-coded in `sort::default_order_with`
(`ready` → `priority` → `status` → `due` → `id`). Promote the chain to a
`[query]` config key so a project can reorder or trim it, and keep the current
chain as the shipped default when the key is absent.

== Motivation

The default chain is one opinion: ready work first, then priority. A deadline-
driven backlog wants `due` to lead; a pure triage backlog wants `priority` only.
The `--sort` flag already covers ad-hoc reordering, but the bare `list` (and the
handle view it must match) is fixed. One config key closes that gap without a new
flag per project.

== Scope

- New `[query] default_sort` key: an ordered list of sort specs, same grammar as
  `--sort` (`field` or `field:asc` / `field:desc`), validated against the known
  sort fields (`id`, `title`, `status`, `priority`, `difficulty`, `kind`, `due`,
  `start`, `file`, `ready`) at config load — a bad field is a config error, not a
  per-command surprise.
- Ship default: when the key is absent, fall back to the current
  `ready` → `priority` → `status` → `due` → `id` chain unchanged, so existing
  projects see no behaviour change.
- `list` applies `default_sort` when `--sort` is empty; an explicit `--sort` still
  wins. Handle resolution (`util::default_view`) threads the same key so list
  ordinals stay in lock-step with what `list` prints.
- Decide the seam: either the CLI expands `default_sort` into the existing
  `sort_tasks` spec path, or `sort::default_order_with` grows a config-driven
  variant. Prefer reusing the `--sort` comparator so one code path orders both.
- Readiness stays a first-class sort field (`ready`), so `default_sort = ["ready",
  "priority"]` reproduces the shipped default explicitly.

== Notes

The hard-coded chain currently lives in `crates/mindtape/src/sort.rs`; the
`--sort` comparator and field validation live in `crates/mindtape-cli/src/list.rs`.
Unifying on one comparator removes the duplication between them.
