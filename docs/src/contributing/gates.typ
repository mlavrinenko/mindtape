// docs/src/contributing/gates.typ renders docs/contributing/gates.md through
// docs/generate-md.sh (typlite; docs/src/lib.typ lists the constraints).
//
// How `just check` runs its gates and what keeps them honest. No doc lists
// the gates. This page shows the commands that print them. The formatter
// path lists below read www/generated/gates.json, which takes them from the
// Justfile's `*_FMT_PATHS` assignments, so a new list renders without an edit
// here.
#import "../lib.typ": doc-title, fact, just

#metadata((
  output: "docs/contributing/gates.md",
)) <mindtape-md>

#doc-title[Quality gates]

How #just.check() runs its gates, what stops two gates from breaking each
other, and what the pre-commit hook asserts. The rules you follow are in
CONTRIBUTING.md.

= How check runs

#just.check() runs its gates in three stages, in order, and stops at the
first stage with a failing gate. The fast stage builds nothing, so you see a
lint or prose error in seconds. The build stage holds every gate that builds
or runs `mt`, and they share one build. The last stage runs #just.cover() and
then #just.crap(), because coverage pays for an instrumented clean rebuild.
Within a stage the gates run in parallel. To see the gates, ask `just` and
mmz rather than a doc:

// docs:hand-typed the commands that list the gates, run against the live Justfile
```bash
just --show _check-stages      # the stage recipes, in order
just --show _check-fast        # the gates in one stage
just --list --list-submodules  # every gate under [gate], with what it checks
mmz --status --tag gate        # which gates are fresh, stale or failed
```

Each arm is #just.mmz("<gate>"), which runs `mmz just <gate>`, so mmz caches
each gate under the recipe's name. mmz skips an arm whose declared inputs
haven't changed since its last pass. `.mmz/config.yaml` declares the inputs
and tags every gate `gate`. Then `mmz --is-fresh --tag gate` asserts that
every gate passed against the current tree, without running any of them. The
pre-commit hook asserts it, and so does `mt flip` into a terminal status,
through an `[[on.flip]]` hook.

A passing #just.check() prints one line, such as
`check: 28 gates passed (3 run, 25 cached)`. An agent usually reads the
output once the run ends, and every line costs it context. A gate's progress
and its report on a clean tree tell that reader nothing a pass doesn't. A
failing run prints the full log of each failed gate, then a verdict naming
them, such as `check: 1 of 28 gates failed: just clippy`, and exits non-zero.
The verdict ends with `(10 not run)` when a red stage stopped the run before
later stages.

`.just/scripts/mmz-arm.sh` does the capture once for every arm, so no tool
needs a quiet flag. It writes each arm's output to `target/check/<gate>.log`,
and the log stays there after a pass. It asks `mmz --is-fresh` first, to count
the arm as run or cached, because a cache hit prints nothing (`on_hit: ""` in
`.mmz/config.yaml`). Outside #just.check(), #just.mmz("<gate>") streams its
output. #just.test("<filter>") runs outside mmz, so a filtered run never
records a pass for the full suite.

Every rule reads the `gates-meta` scope, which includes the `Justfile`, so a
`Justfile` edit re-runs every arm. Each script under `.just/scripts/` has a
scope of its own, so editing one re-runs the arms that reach it and the few
that read the whole directory.

== Inputs a glob can't name

#just.crap() reads `target/coverage/lcov.info`, which git ignores. mmz drops
ignored paths from a scope, so the `lcov` scope sets `gitignore: false`. With
the lcov in its inputs, the arm re-runs when #just.cover() writes a new one.

#just.check-secrets() scans the whole tree, but `.mmz/config.yaml` has no
negative glob, and no gate may name `truth/tasks/**`. `mt flip` writes the
task file it gates, so such an input would make the gate stale in the middle
of the flip. So the arm's inputs are one glob per top-level directory, and
`.just/scripts/check-secrets-manifest.sh` fails naming any tracked file those
globs miss.

== Arms must not read what another arm writes

The arms share one working tree. If an arm reads files another arm writes, it
has to stay out of them or take the writer's lock.

- `www/generate.sh` deletes and recreates `www/generated/` on every run. Every
  reader and writer of that directory holds the `target/www-generated.lock`
  flock for its whole run. The readers regenerate through
  `mmz just docs::gen` inside that lock, so the generator runs once per tree.
- #just.site-pages-fixture-check() copies `www/` into a temporary replica. It
  skips `public/` and `.tola/`, which the #just.docs-check() arm writes, and
  regenerates `www/generated/` inside the lock.
- #just.check-secrets() derives a gitleaks allowlist from the paths git
  ignores, so the scan skips `www/generated/` and `www/.tola/`. gitleaks
  still stats the skipped directory itself, so the scan holds the lock too.
  `.just/scripts/check-secrets.sh` has the measurements.
- mmz hashes a rule's inputs before it runs the rule, and aborts with exit 8
  if a file vanishes during the hash. That's why `www/.gitignore` ignores the
  temporary files the favicon conversion writes, and both mmz and the
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
as #just.build-mt() does with `target/mt-dev`.

#just.check-gate-drift() fails naming file and line on any build of
`mindtape-cli` that a #just.check() arm reaches without `--tests` or a flag
that writes elsewhere.

== Why each script has its own scope

`.mmz/config.yaml` names one scope per file under `.just/scripts/`. A single
glob over the directory would re-run every gate when one script changed. A
list covers only the scripts someone added to it, so
#just.script-scope-check() fails on a script no scope names, on a rule that
reaches a script its inputs miss, and on a script that's a source of no
`outdatty.yaml` group. Put a new script in the group of the doc that restates
it, or in the source-only group when no doc does.
`.just/scripts/script-scope-check.sh` explains each arm.

= The formatter path lists

#just.fmt() and #just.fmt-check() read one list per formatter from the
`Justfile`, so a path you add there reaches both. #just.check-gate-drift()
fails when `TOML_FMT_PATHS` misses a TOML file git lists.

#for (name, value) in fact("gates").at("assignments") [
  - #raw(name): #value.split(" ").map(raw).join(", ")
]

just's `--fmt` flag alone formats the root `Justfile` and skips every `mod`
file (verified on just 1.46). So each file in `JUST_FMT_PATHS` gets its own
`--justfile <file>` pass, `--fmt` in #just.fmt() and `--dump` diffed against
the file in #just.fmt-check(), and each needs its own `set unstable := true`.
If a module file moves without its entry, it drops out of both recipes.

= What the pre-commit hook asserts

`.githooks/pre-commit` runs `mmz --is-fresh --tag gate`, then
`.githooks/local-paths`, `typstyle --check truth` and `mt check`. It refuses
the commit when any of them fails.

`.githooks/local-paths` refuses a staged file that names your `$HOME`, the
checkout or the primary checkout. Such a path rides in on a pasted error or
log, and only your machine knows it's local, so no gate in CI can catch it.
Write `<repo>`, `<worktree>` or `~` in its place. `.githooks/commit-msg` runs
the same scan over the message.

`MINDTAPE_GATE_SCOPE=<reason>` skips the first assertion only. The other three
still run, and `.githooks/commit-msg` refuses the commit unless its message
carries a matching `Gate-Scope: <reason>` trailer. So `git log` shows which
commits skipped the gate. CONTRIBUTING.md says when you may use it.
