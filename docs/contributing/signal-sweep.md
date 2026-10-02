<!-- Generated from docs/src/contributing/signal-sweep.typ by `just docs md`. Do not edit; edit the source. -->

# Signal sweep

`just ask-refactoring` runs `contrib/ask-refactoring.sh`, which reads the quality signals no gate enforces and turns each finding into a backlog task. A gate fails a commit. The sweep files tasks for signals that fail nothing yet, such as a file at 90% of its size cap.

By default it prints the tasks it would file. With `--write` it files them. A threshold flag, such as `--coverage-floor 92`, changes that arm for one run.

| Flag | Default | Bounds | Notes |
| --- | --- | --- | --- |
| `--write` | `0` | file the tasks instead of printing what would be filed | The one flag that is not a threshold: it decides whether a finding is printed or filed, never which findings there are. Dry is the default because the sweep is a proposal — the remedy is the refactorer’s call, and a proposal you have to read before it becomes a task is how that stays true. |
| `--baseline BASELINE` | `90` | size arm: report files at or above PCT% of their linecop limit | A percentage of each file’s own cap, not a line count: the caps differ per language and per override, so the one number that means the same thing everywhere is how much of a file’s own budget is spent. Sits above linecop’s own `--baseline 85` reporting band, so the sweep files a task strictly later than the number a contributor is told to watch. |
| `--coverage-floor COVERAGE_FLOOR` | `88` | coverage arm: report files below PCT% line coverage | Per-file line coverage, and the only arm measuring something no gate measures: `just cover` fails on the workspace total alone, so a file can sink without any gate noticing. Its own band, and deliberately not the workspace gate’s number — reusing that would imply the two mean the same thing. |
| `--cc-floor CC_FLOOR` | `5` | undertested arm: floor on cyclomatic complexity | Half of a conjunction with `--fn-coverage-floor`, and the half that is not a complexity judgement — `just crap` gates complexity, at its own threshold against a committed baseline. This floor is what makes the coverage number mean something: below it, a low percentage says one line is unhit; at or above it, it says decisions are unexercised. |
| `--fn-coverage-floor FN_COVERAGE_FLOOR` | `80` | undertested arm: report functions below PCT% line coverage | The other half, per function rather than per file, and stricter than `--coverage-floor` on purpose: a function is small enough that one unreached line moves its percentage by tens of points, so the tail starts further out. Two edges rather than one shared number is the point of the pair — a function has to be both complex enough and undertested enough — and the undertested arm defers to the coverage arm on any file that one already reports, so the overlap files nothing twice. |

The defaults live in the recipe’s `[arg(...)]` attributes and nowhere else. `contrib/ask-refactoring.sh` requires all five flags and defaults none, and just’s `--usage` flag prints the same set. `--write` takes no argument. Passing it sets `1`.

The sweep has four thresholds, one per measurement, because a flag shared by two arms would move both at once. The undertested arm takes two, because a function must be both complex and undertested to be worth a task. Each number’s argument sits beside its constant in the arm script that applies it.

The mutants arm has no threshold. It reads `outcomes.json` from whichever sweep ran last: `mutants.out` after a full `just mutants`, or `target/mutants-scoped` after a scoped run. When neither exists, or the one it picked is stale, it skips with a message naming the command that refreshes it. `just mutants-status` reports a running sweep’s progress and ETA.

## How it is built

`contrib/ask-refactoring.sh` holds the flag handling, the re-run guard, the shared `stem_for` and the driver. Each arm is one file under `contrib/ask-refactoring/` that defines `score_<arm>`, `signal_<arm>` and `emit_<arm>`. A new arm is a file there, its name in `ARMS`, and a `.mindtape/templates/sweep-*.typ` template. The tags and the guard are declared already, so it needs no config change.

The templates share one shape, and the `sweep-templates` outdatty group keeps them in step. A Typst `#import` would end up in every task the sweep writes and tie a tracked task file to a path outside the tasks root.

## The design rules

- Only signals that no gate enforces qualify. jscpd, outdatty, machete and clippy already fail a commit, so a task from one would repeat that failure.
- An arm never reports a measurement it does not have. An arm that can refresh its input refuses a stale one. An arm that cannot, such as the hours-long mutation sweep, skips with a message naming the refresh command. A missing measurement never reads as a clean one.
- One task per finding. Each task goes to its own agent and carries its own scope and remedy.
- No two arms file the same finding. When two arms read the same data, the finer arm calls the coarser arm’s own predicate (`coverage_arm_files`, kept in step by the `sweep-dedup` outdatty group). A duplicate pair would block the next sweep until both close.
- The signal decides what is filed again, and no ledger records past findings. To silence a finding for good, write an override with its reason in the signal’s own config.
- The remedy is the refactorer’s call. Each template names two correct answers: do the work, or write the override that says why not.

The sweep is not an mmz arm and not part of `just check`. It has to read the tree fresh on every run, and a cached result is the one answer it must never give.

`[views.ask-refactoring]` in `.mindtape/config.toml` (`has_tag("swept") && !terminal`) is the re-run guard. While a task the sweep filed is still open, the sweep refuses and prints the count, so one batch of mechanical work is done before the next is filed. Every filed task carries the tag `swept` and one tag naming the tool that found it. The guard selects on `swept` alone, so a new arm needs no change to it.
