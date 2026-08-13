// docs/src/contributing/signal-sweep.typ — source for
// docs/contributing/signal-sweep.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// Content-preserving port of the hand-written signal-sweep.md, including its
// heading level: the original has no h1, only a single top-level `##`, so
// this source opens at Typst's `=` (one level below `doc-title`) rather than
// importing `doc-title` the way every other source here does. Nothing is
// derived: the sweep's arms, its rules and its guard expression are design,
// not values any of www/generate-facts.sh's four artifacts carries.
#import "../lib.typ": just

#metadata((
  output: "docs/contributing/signal-sweep.md",
)) <mindtape-md>

= Signal Sweep (#just.ask-refactoring())

`contrib/ask-refactoring.sh` reads the _advisory_ quality signals and files a
backlog task per finding. It is not a gate and never will be — it is the
other half of the gates: a gate stops you when the wall is already in front
of you, the sweep says which wall you are walking toward.

```
just ask-refactoring          # dry: print the tasks it would file
just ask-refactoring --write  # file them
just ask-refactoring --help   # each arm's own threshold flag, with its default
```

The entry script holds the argument parsing, the re-run guard, the shared
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

The re-run guard is `[views.ask-refactoring]` in `.mindtape/config.toml`
(`has_tag("swept") && !terminal`). While anything the sweep filed is still
open, the sweep refuses and names the count — one sprint of mechanical work
is emptied before the next is filed. Every filed task carries the provenance
tag `swept` plus one signal tag naming the tool that noticed — two arms
reading one tool share it — and the guard selects on `swept` alone, so a new
arm joins for free.
