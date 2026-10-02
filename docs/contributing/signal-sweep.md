<!-- Generated from docs/src/contributing/signal-sweep.typ by `just docs md`. Do not edit; edit the source. -->

## Signal Sweep (`just ask-refactoring`)

`contrib/ask-refactoring.sh` reads the _advisory_ quality signals and files a backlog task per finding. It is not a gate and never will be — it is the other half of the gates: a gate stops you when the wall is already in front of you, the sweep says which wall you are walking toward.

```
just ask-refactoring                      # dry: print the tasks it would file
just ask-refactoring --write              # file them
just ask-refactoring --coverage-floor 92  # one arm's threshold, this run only
```

| Flag | Default | Bounds | Notes |
| --- | --- | --- | --- |
| `--write` | `0` | file the tasks instead of printing what would be filed | The one flag that is not a threshold: it decides whether a finding is printed or filed, never which findings there are. Dry is the default because the sweep is a proposal — the remedy is the refactorer’s call, and a proposal you have to read before it becomes a task is how that stays true. |
| `--baseline BASELINE` | `90` | size arm: report files at or above PCT% of their linecop limit | A percentage of each file’s own cap, not a line count: the caps differ per language and per override, so the one number that means the same thing everywhere is how much of a file’s own budget is spent. Sits above linecop’s own `--baseline 85` reporting band, so the sweep files a task strictly later than the number a contributor is told to watch. |
| `--coverage-floor COVERAGE_FLOOR` | `88` | coverage arm: report files below PCT% line coverage | Per-file line coverage, and the only arm measuring something no gate measures: `just cover` fails on the workspace total alone, so a file can sink without any gate noticing. Its own band, and deliberately not the workspace gate’s number — reusing that would imply the two mean the same thing. |
| `--cc-floor CC_FLOOR` | `5` | undertested arm: floor on cyclomatic complexity | Half of a conjunction with `--fn-coverage-floor`, and the half that is not a complexity judgement — `just crap` gates complexity, at its own threshold against a committed baseline. This floor is what makes the coverage number mean something: below it, a low percentage says one line is unhit; at or above it, it says decisions are unexercised. |
| `--fn-coverage-floor FN_COVERAGE_FLOOR` | `80` | undertested arm: report functions below PCT% line coverage | The other half, per function rather than per file, and stricter than `--coverage-floor` on purpose: a function is small enough that one unreached line moves its percentage by tens of points, so the tail starts further out. Two edges rather than one shared number is the point of the pair — a function has to be both complex enough and undertested enough — and the undertested arm defers to the coverage arm on any file that one already reports, so the overlap files nothing twice. |

Every row above is read out of the `ask-refactoring` recipe’s own `[arg(...)]` parameters, so the defaults live in the Justfile and nowhere else: `contrib/ask-refactoring.sh` requires all five and defaults nothing, and just’s own `--usage` flag prints the same set at the terminal, from the same data. `--write` takes no argument of its own: passing it sets `1`.

The sweep carries four thresholds rather than one band shared across the arms, and that is a decision rather than an accident: one flag over two arms would read as a single number and move two at once. The pair the undertested arm carries is a conjunction — a function has to be both complex enough and undertested enough to be worth a task — and the size and coverage bands belong to different arms measuring different things. Each number’s full argument lives in the arm script that applies it, next to the constant rather than in a doc that would drift from it.

The mutants arm carries no threshold at all: it reads the outcomes.json of whichever sweep ran last — `mutants.out` for a full `just mutants`, or `target/mutants-scoped` for a `-f`/`--in-diff` run — and skips loudly when neither exists or the one it picked is stale, rather than run a sweep that takes hours. `just mutants-status` reports a running sweep’s progress and ETA from the directory it is given (`mutants.out` by default).

The entry script holds the flag forwarding, the re-run guard, the shared `stem_for` and the driver. Each arm is one file under `contrib/ask-refactoring/`, defining `score_<arm>`/`signal_<arm>`/`emit_<arm>` next to the reasoning that justifies its numbers. Adding an arm is a new file there, its name in `ARMS`, and a `.mindtape/templates/sweep-*.typ` template — never a config change, since the tags and the guard are already declared. The templates share one shape, kept in step by the `sweep-templates` outdatty group rather than by a Typst `#import`, which would survive into every task the sweep writes and bind a tracked artifact to a path outside the tasks root. Which arms exist, what each reads and how each scores are the scripts’ own subject.

The rules the design turns on, in the order they matter:

- Only advisory signals qualify. jscpd, outdatty, machete and clippy already block a commit, so a task from one would be a note about that wall.
- A signal never fakes a measurement it does not have. An arm that can refresh its input refuses a stale one; an arm that cannot — a full mutation sweep is hours — skips loudly, naming the command that would refresh it, rather than blocking the other arms or printing “clean” when nobody has measured. Absent and clean must never look alike.
- One task per finding, never one per sweep. Each is dispatched to its own agent and has to carry its own scope, including which remedy applies to it.
- No two arms file the same finding. Granularities overlap — a per-file and a per-function arm read one lcov — so the finer defers by calling the coarser arm’s own predicate, not a copy (`coverage_arm_files`, plus the `sweep-dedup` outdatty group). A duplicate pair blocks the next sweep until both halves close: twice the work for one gap.
- Idempotency comes from the signal, not a ledger. Nothing records what was filed, so the only way to silence a finding for good is an override with a written reason in the signal’s own config. There is no ignore list anywhere else, deliberately: silencing always leaves a sentence behind saying why.
- The remedy is the refactorer’s call. Every arm offers two correct answers — do the work, or write the override saying why not — and each template names its own pair. The signal pushes back on drift, never on accuracy.

It is deliberately not an `mmz` arm and not part of `just check`. Memoizing it would defeat the point: it has to re-observe the tree every run, and a cache hit is the one answer it must never give.

The re-run guard is `[views.ask-refactoring]` in `.mindtape/config.toml` (`has_tag("swept") && !terminal`). While anything the sweep filed is still open, the sweep refuses and names the count, so one sprint of mechanical work is emptied before the next is filed. Every filed task carries the provenance tag `swept` plus one signal tag naming the tool that noticed — two arms reading one tool share it — and the guard selects on `swept` alone, so a new arm joins for free.
