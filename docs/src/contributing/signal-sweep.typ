// docs/src/contributing/signal-sweep.typ renders
// docs/contributing/signal-sweep.md through docs/generate-md.sh (typlite;
// docs/src/lib.typ lists the constraints).
//
// The flag table reads www/generated/sweep-flags.json, which takes the flags
// and their defaults from the `ask-refactoring` recipe's `[arg(...)]`
// attributes. The Notes column comes from www/utils/sweep-flag-notes.typ,
// keyed by flag. The counts in the prose come from the same file.
#import "../lib.typ": doc-title, fact, just, num-word
#import "/www/utils/sweep-flag-notes.typ": sweep-flag-notes

#metadata((
  output: "docs/contributing/signal-sweep.md",
)) <mindtape-md>

#doc-title[Signal sweep]

To turn the quality signals no gate enforces into backlog tasks, run
#just.ask-refactoring(). It runs `contrib/ask-refactoring.sh`, which files one
task per finding. A gate fails a commit. The sweep files tasks for signals
that fail nothing yet, such as a file at 90% of its size cap.

By default it prints each arm's finding count and its three findings with the
highest ICE score, and writes those tasks' bodies to `target/ask-refactoring/`,
so you can read them before you file. Add `--write` to file every finding. A
threshold flag, such as `--coverage-floor 92`, changes that arm for one run.

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
#let thresholds = sweep-flags.filter(f => f.value == none)
The defaults live in the recipe's `[arg(...)]` attributes and nowhere else.
`contrib/ask-refactoring.sh` requires all #num-word(sweep-flags.len()) flags
and defaults none, and just's `--usage` flag prints the same set.
#for f in fixed-value [
  #raw("--" + f.long) takes no argument. Passing it sets #raw(f.value).
]

The sweep has #num-word(thresholds.len()) thresholds, one per measurement,
because a flag shared by two arms would move both at once. The undertested
arm takes two, because a function must be both complex and undertested to be
worth a task. Each number's argument sits beside its constant in the arm
script that applies it.

The repeats arm is a stopgap. It groups jscpd's clone pairs by hand into
snippets pasted many times, and files the ones whose extraction saves the most
code. Once jscpd groups clones itself, its blocker in `canibump.toml` reads
`bumpable` and the arm goes.

The dry arm files the clones #just.check-dry() tolerates. It runs jscpd with
the gate's flags, keeps the pairs the baseline already holds, and joins them
into one family per shared location. It files a family only when extracting
it saves `--dry-gain` tokens or more. A task closes by extracting the clone, or by a `jscpd:ignore`
marker whose comment says why the match is a coincidence. Either way the
commit re-mints the baseline with #just.dry-baseline().

The mutants arm has no threshold. It reads `outcomes.json` from whichever
sweep ran last: `mutants.out` after a full #just.mutants(), or
`target/mutants-scoped` after a scoped run. When neither exists, or the one
it picked is stale, it skips with a message naming the command that
refreshes it. #just.mutants-status() reports a running sweep's progress and
ETA.

= How it's built

`contrib/ask-refactoring.sh` holds the flag handling, the re-run guard, the
shared `stem_for` and the driver. Each arm is one file under
`contrib/ask-refactoring/` that defines `score_<arm>`, `signal_<arm>` and
`emit_<arm>`. To add an arm, add a file there, its name in `ARMS`, and a
`.mindtape/templates/sweep-*.typ` template. The tags and the guard are
declared already, so it needs no config change.

The templates share one shape, and the `sweep-templates` outdatty group keeps
them in step. They don't share it through a Typst `#import`, because the
import would end up in every task the sweep writes and tie a tracked task
file to a path outside the tasks root.

= The design rules

- Only signals that no gate enforces qualify. outdatty, machete and clippy
  already fail a commit, so a task from one would repeat that failure. jscpd
  fails a new clone only from 50 tokens up, so the repeats arm reads it below
  that and the dry arm reads the old clones its baseline tolerates.
- An arm never reports a measurement it doesn't have. An arm that can
  refresh its input refuses a stale one. An arm that can't, such as the
  hours-long mutation sweep, skips with a message naming the refresh command.
  A missing measurement never reads as a clean one.
- One task per finding. Each task goes to its own agent and carries its own
  scope and remedy.
- No two arms file the same finding. When two arms read the same data, the
  finer arm calls the coarser arm's own predicate (`coverage_arm_files`, kept
  in step by the `sweep-dedup` outdatty group). A duplicate pair would block
  the next sweep until both close.
- The signal decides what gets filed again, and no ledger records past
  findings. To silence a finding for good, write an override with its reason
  in the signal's own config.
- The remedy is the refactorer's call. Each template names two correct
  answers: do the work, or write the override that says why not.

The sweep isn't an mmz arm or part of #just.check(). It has to read the tree
fresh on every run, and a cached result is the one answer it must never give.

`[views.ask-refactoring]` in `.mindtape/config.toml`
(`has_tag("swept") && !terminal`) is the re-run guard. While a task the sweep
filed is still open, the sweep refuses and prints the count, so you finish one
batch of mechanical work before the next is filed. The sweep files into
`truth/tasks/swept/`, so a finished batch goes with
`mt rm 'has_tag("swept") && terminal'`. Every filed task carries
the tag `swept` and one tag naming the tool that found it. The guard selects
on `swept` alone, so a new arm needs no change to it.
