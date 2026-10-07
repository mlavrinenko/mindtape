// docs/src/contributing/gates.typ renders docs/contributing/gates.md through
// docs/generate-md.sh (typlite; docs/src/lib.typ lists the constraints).
//
// How `just check` runs its gates and what keeps them honest. No doc lists
// the gates; this page shows the commands that print them. The formatter path
// lists below read
// www/generated/gates.json, which takes them from the Justfile's
// `*_FMT_PATHS` assignments, so a new list renders without an edit here.
#import "../lib.typ": doc-title, fact, just

#metadata((
  output: "docs/contributing/gates.md",
)) <mindtape-md>

#doc-title[Quality gates]

How #just.check() runs its gates, what stops two gates from breaking each
other, and what the pre-commit hook asserts. The rules for contributors are in
CONTRIBUTING.md.

= How check runs

#just.check() runs its gates in three stages, in order, and stops at the
first stage with a failing gate. The fast stage builds nothing, so a lint or
prose error shows in seconds. The build stage holds every gate that builds or
runs `mt`, and they share one build. The last stage runs #just.cover() and
then #just.crap(), because coverage pays an instrumented clean rebuild.
Within a stage the gates run in parallel. Ask `just` and mmz for the gates
rather than a doc:

// docs:hand-typed the commands that list the gates, run against the live Justfile
```bash
just --show _check-stages      # the stage recipes, in order
just --show _check-fast        # the gates in one stage
just --list --list-submodules  # every gate under [gate], with what it checks
mmz --status --tag gate        # which gates are fresh, stale or failed now
```

Each arm is #just.mmz("<gate>"), which runs `mmz just <gate>`, so mmz caches
each gate under the recipe's name. An arm whose declared inputs are unchanged since its last pass is
skipped. `.mmz/config.yaml` declares the inputs and tags every gate `gate`.
Then `mmz --is-fresh --tag gate` asserts that every gate passed against the
current tree without running any of them. The pre-commit hook asserts it,
and so does `mt flip` into a terminal status, through an `[[on.flip]]` hook.

A passing #just.check() prints one line, such as
`check: 28 gates passed (3 run, 25 cached)`. Mostly an agent reads the
output, once the run has ended, and every line it reads costs context. A
gate's progress and its report on a clean tree tell that reader nothing a
pass does not. A failing run prints the full log of each failed gate, then a
verdict that names them, such as `check: 1 of 28 gates failed: just clippy`,
and exits non-zero. A verdict ends with `(10 not run)` when a red stage
stopped the run before later stages.

`.just/scripts/mmz-arm.sh` does the capture, once for every arm, so no tool
needs a quiet flag. It writes each arm's output to
`target/check/<gate>.log`, and that log stays there after a pass. It asks
`mmz --is-fresh` first, to count the arm as run or cached, because a cache
hit prints nothing (`on_hit: ""` in `.mmz/config.yaml`). Outside
#just.check(), #just.mmz("<gate>") streams its output as before.
#just.test("<filter>") runs outside mmz, so a filtered run never records a
pass for the full suite.

Every rule reads the `gates-meta` scope, which includes the `Justfile`, so a
`Justfile` edit re-runs every arm. A script under `.just/scripts/` has a
scope of its own, so editing it re-runs the arms that reach it and the few
that read the whole directory.

== Inputs a glob cannot name

#just.crap() reads `target/coverage/lcov.info`, which git ignores. Its
`lcov` scope sets `gitignore: false`, because mmz drops ignored paths from
every other scope. With the lcov in its inputs, the arm reruns when
#just.cover() writes a new lcov.

#just.check-secrets() scans the whole tree, but `.mmz/config.yaml` has no
negative glob, and no gate may name `truth/tasks/**`. `mt flip` writes the
task file it gates, so such an input would make the gate stale in the middle
of the flip. The arm's inputs are therefore one glob per top-level
directory, and `.just/scripts/check-secrets-manifest.sh` fails naming any
tracked file those globs miss.

== Arms must not read what another arm writes

The arms share one working tree. An arm that reads files another arm writes
must stay out of them or take the writer's lock.

- `www/generate.sh` deletes and recreates `www/generated/` on every run. Every
  reader and writer of that directory holds the `target/www-generated.lock`
  flock for its whole run. The readers regenerate through
  `mmz just docs::gen` inside that lock, so the generator runs once per tree.
- #just.site-pages-fixture-check() copies `www/` into a temporary replica. It
  skips `public/` and `.tola/`, which the #just.docs-check() arm writes, and
  regenerates `www/generated/` inside the lock.
- #just.check-secrets() derives a gitleaks allowlist from the paths git
  ignores, so the scan skips `www/generated/` and `www/.tola/`. gitleaks
  still stats the skipped directory itself, so the scan also holds the lock.
  `.just/scripts/check-secrets.sh` has the measurements.
- mmz hashes a rule's inputs before it runs the rule, and aborts with exit 8
  if a file vanishes during the hash. `www/.gitignore` ignores the temporary
  files the favicon conversion writes for that reason, and both mmz and the
  gitleaks bound read that file.

== One mt binary, one feature set

The #just.test() arm runs `target/debug/mt` (`CARGO_BIN_EXE_mt`), and every
arm that needs `mt` builds it. `mindtape-cli`'s dev-dependencies turn on the
`test-support` features of `mindtape` and `swhid-mint`. So
`cargo test --no-run` and a plain `cargo build -p mindtape-cli` build two
different binaries for the same path. Cargo hard-links the newest build to
`target/debug/mt`, and the swap deletes the file for about 85 ms. A test that
spawns `mt` in that window fails with `No such file or directory`.

So every arm that builds `mt` runs `cargo build -p mindtape-cli --tests`,
which builds the same binary as the test arm. #just.check-examples() builds
that way and runs `./target/debug/mt`, because `cargo run` takes no
`--tests`. A build with other features writes to another target directory,
as #just.build-mt() does with `target/mt-dev`. #just.check-rrule() builds the
`rrule` features into `target/debug` and is safe only because it is not a
#just.check() arm. CI runs it alone.

#just.check-gate-drift() fails naming file and line on any build of
`mindtape-cli` that a #just.check() arm reaches without `--tests` or a flag
that writes elsewhere.

== Why each script has its own scope

`.mmz/config.yaml` names one scope per file under `.just/scripts/`. A single
glob over the directory would re-run every gate when one script changed. A
list only covers the scripts someone added to it, so
#just.script-scope-check() fails on a script no scope names, on a rule that
reaches a script its inputs miss, and on a script that is a source of no
`outdatty.yaml` group. A script joins the group of the doc that restates it, or
the source-only group when no doc does. `.just/scripts/script-scope-check.sh`
explains each arm.

= The formatter path lists

#just.fmt() and #just.fmt-check() read one list per formatter from the
`Justfile`, so a path added there reaches both. #just.check-gate-drift() fails
when `TOML_FMT_PATHS` misses a TOML file git lists.

#for (name, value) in fact("gates").at("assignments") [
  - #raw(name): #value.split(" ").map(raw).join(", ")
]

just's `--fmt` flag alone formats the root `Justfile` and skips every `mod`
file (verified on just 1.46). So each file in `JUST_FMT_PATHS` gets its own
`--justfile <file> --fmt` pass, and each needs `set unstable := true` of its
own. A module file that moves without its entry drops out of both recipes.

= What the pre-commit hook asserts

`.githooks/pre-commit` runs `mmz --is-fresh --tag gate`, then
`typstyle --check truth` and `mt check`. It refuses the commit when any of
them fails.

`MINDTAPE_GATE_SCOPE=<reason>` skips the first assertion only. The other two
still run, and `.githooks/commit-msg` refuses the commit unless its message
carries a matching `Gate-Scope: <reason>` trailer. So `git log` shows which
commits skipped the gate. When a commit may use it is in CONTRIBUTING.md.
