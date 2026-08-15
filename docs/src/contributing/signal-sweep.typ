// docs/src/contributing/signal-sweep.typ — source for
// docs/contributing/signal-sweep.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// Content-preserving port of the hand-written signal-sweep.md, including its
// heading level: the original has no h1, only a single top-level `##`, so
// this source opens at Typst's `=` (one level below `doc-title`) rather than
// importing `doc-title` the way every other source here does.
//
// One thing here IS derived, and it is the one thing on this page that is a
// set of values rather than design: the flag table. It reads
// www/generated/sweep-flags.json — `ask-refactoring`'s own `[arg(...)]`
// parameters, straight out of `just --dump` — so the five thresholds and
// their defaults are stated once, in the Justfile, and rendered here
// (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-attributes.typ).
// The Notes column is the half no machine can state and stays hand-written in
// www/utils/sweep-flag-notes.typ, keyed by flag; a flag missing there fails
// `just docs::md` naming it. The sweep's arms, its rules and its guard
// expression stay design prose, derived from nothing.
#import "../lib.typ": fact, just
#import "/www/utils/sweep-flag-notes.typ": sweep-flag-notes

#metadata((
  output: "docs/contributing/signal-sweep.md",
)) <mindtape-md>

= Signal Sweep (#just.ask-refactoring())

`contrib/ask-refactoring.sh` reads the _advisory_ quality signals and files a
backlog task per finding. It is not a gate and never will be — it is the
other half of the gates: a gate stops you when the wall is already in front
of you, the sweep says which wall you are walking toward.

```
just ask-refactoring                      # dry: print the tasks it would file
just ask-refactoring --write              # file them
just ask-refactoring --coverage-floor 92  # one arm's threshold, this run only
```

#let sweep-flags = fact("sweep-flags").at("flags")
#table(
  columns: 4,
  table.header([Flag], [Default], [Bounds], [Notes]),
  ..sweep-flags
    .map(f => (
      raw("--" + f.long + if f.value == none { " " + f.name } else { "" }),
      raw(f.default),
      f.help,
      sweep-flag-notes.at(f.long),
    ))
    .flatten(),
)

#let fixed-value = sweep-flags.filter(f => f.value != none)
Every row above is read out of the `ask-refactoring` recipe's own
`[arg(...)]` parameters, so the defaults exist in the Justfile and nowhere
else — `contrib/ask-refactoring.sh` requires all five and defaults nothing.
just's own `--usage` flag prints the same five at the terminal, from the same
data. That is a change rather than how it always was: the five used to be a
paragraph of Justfile comment plus a parse loop with its own defaults in
`contrib/ask-refactoring.sh`, so nothing rendered them, the `--list` output
showed none of them, and the two copies could disagree without anything
noticing
(`truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-attributes.typ`).
#if fixed-value.len() > 0 [
  #for f in fixed-value [
    #raw("--" + f.long) takes no argument of its own: passing it sets
    #raw(f.value).
  ]
]

Four thresholds, never one band shared across the arms, and that is a
decision rather than an accident: one flag over two arms would read as a
single number and move two at once. The pair the undertested arm carries is a
conjunction, so it is two edges rather than one — a function has to be both
complex enough and undertested enough to be worth a task — and the size and
coverage bands belong to different arms measuring different things. Each
number's full argument (the distributions behind it, what it rejects, why it
stops where it does) lives in the arm script that applies it, next to the
constant rather than in a doc that would drift from it.

The entry script holds the flag forwarding, the re-run guard, the shared
`stem_for` and the driver; each arm is one file under
`contrib/ask-refactoring/`, defining `score_<arm>`/`signal_<arm>`/`emit_<arm>`
next to the reasoning that justifies its numbers. Adding an arm is a new file
there, its name in `ARMS`, and a `.mindtape/templates/sweep-*.typ` template —
never a config change, since the tags and the guard are already declared. The
templates share one shape, kept in step by the `sweep-templates` outdatty
group rather than by a Typst `#import`, which would survive into every task
the sweep writes and bind a tracked artifact to a path outside the tasks
root. Which arms exist, what each reads and how each scores are the scripts'
own subject — read them there rather than here.

Six rules the design turns on, in the order they matter:

- Only advisory signals qualify. jscpd, outdatty, machete and clippy already
  block a commit, so a task from one would be a note about that wall.
- A signal never fakes a measurement it does not have. An arm that can
  refresh its input refuses a stale one; an arm that cannot — a full mutation
  sweep is hours — skips loudly, naming the command that would refresh it,
  rather than blocking the other arms or printing "clean" when nobody has
  measured. Absent and clean must never look alike.
- One task per finding, never one per sweep. Each is dispatched to its own
  agent and has to carry its own scope, including which remedy applies to it.
- No two arms file the same finding. Granularities overlap — a per-file and a
  per-function arm read one lcov — so the finer defers by calling the
  coarser arm's own predicate, not a copy (`coverage_arm_files`, plus the
  `sweep-dedup` outdatty group). Exactness matters because a duplicate pair
  blocks the next sweep until both halves close: twice the work for one gap.
- Idempotency comes from the signal, not a ledger. Nothing records what was
  filed, so the only way to silence a finding for good is an override with a
  written reason in the signal's own config. There is no ignore list anywhere
  else, deliberately: silencing always leaves a sentence behind saying why.
- The remedy is the refactorer's call. Every arm offers two correct answers —
  do the work, or write the override saying why not — and each template
  names its own pair. The signal pushes back on drift, never on accuracy.

It is deliberately not an `mmz` arm and not part of #just.check() either.
Memoizing it would defeat the point: it has to re-observe the tree every run,
and a cache hit is the one answer it must never give.

The re-run guard is `[views.ask-refactoring]` in `.mindtape/config.toml`
(`has_tag("swept") && !terminal`). While anything the sweep filed is still
open, the sweep refuses and names the count — one sprint of mechanical work
is emptied before the next is filed. Every filed task carries the provenance
tag `swept` plus one signal tag naming the tool that noticed — two arms
reading one tool share it — and the guard selects on `swept` alone, so a new
arm joins for free.
