#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mmz: per-file-command cache via glob-fanned rules",
  priority: ice(
    impact: 5,
    confidence: 0.5,
    ease: 4,
  ),
  tags: ("tooling",),
  status: proposed(2026, 7, 17),
)

== Summary

mmz keys a cache record by the matched rule `name`, so memoizing a per-file
command (`cargo mutants -f <file>`, a per-file lint, ...) needs one `match:
exact` rule per file. That hand-list drifts from the source tree and is pure
boilerplate. Add a rule form that fans a command template over a file-set glob:
one declaration yields one cache record per matched file, each keyed by that
file and scoped (inputs) to that file plus shared pins.

== Why

Discovered while wiring mutation testing for mindtape. The per-file mmz approach
worked but required hand-listing every target as its own rule + scope —
guaranteed to drift from `./src`. cargo-mutants' native `--iterate` +
`--list-files` covered the mindtape mutants case without mmz, so mmz is off that
critical path; the underlying gap is general, though — any per-file-command
memoization hits it.

== Shape (sketch)

- A rule gains an optional file-set glob that expands at load into N logical
  rules, one per matched file.
- Cache identity = rule name + the file, so records are per-file, not shared.
- The matched file is injected into the command template and into that record's
  inputs, so a file busts only its own record (tight per-file invalidation) —
  the property the static one-rule-per-file form gives, minus the boilerplate.
- `--status` enumerates the expanded per-file records.

Design tension: this leans toward per-file input derivation, near the
"no dependency tracing" non-goal. Keep it a literal glob-fan (still fully
declared, no inference), not inferred deps.

== Home

Belongs in mmz's own backlog. mmz is not yet a MindTape project; captured here
at the discovery point until mmz adopts `mt`. Relocate on bootstrap.
