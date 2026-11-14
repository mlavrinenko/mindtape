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

The rules you follow are in CONTRIBUTING.md.

= How check runs

#just.check() runs its gates in three stages, in order, and stops at the
first stage with a failing gate. The fast stage builds nothing, so you see a
lint error in seconds. The build stage contains every gate that builds
or runs `mt`, and they share one build. The last stage runs #just.cover() and
then #just.crap-check(), because coverage pays for an instrumented clean rebuild.
Within a stage the gates run in parallel. Name stages to run only those, such
as #just.check("fast build"). CI does that to run `cover` in a job of its own,
because it doesn't share a build with the other two. To see the gates, ask `just` and
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
haven't changed since its last pass. `.mmz/config.yaml` declares the inputs,
and tags a rule `gate` when its own writes cannot stale its record. Then
`mmz --is-fresh --tag gate` asserts that every tagged rule passed against the
current tree, without running any of them. The pre-commit hook asserts it, and
so does `mt flip` into a terminal status, through an `[[on.flip]]` hook. One
arm, #just.docs-vale-tasks(), carries no tag, and "Inputs a plain glob can't
name" below says why.

A passing #just.check() prints one line, such as
`check: <n> gates passed (<ran> run, <cached> cached)`. An agent reads
the output once the run ends, and every line costs it context. For a reader
that only sees the final output, a gate's progress lines and clean-tree report
add nothing to the pass summary. A failing run prints the full log of each failed gate, then a verdict
naming them, such as `check: 1 of <n> gates failed: just clippy`, and exits
non-zero. The verdict ends with `(10 not run)` when a red stage stopped the run
before later stages.

`.just/scripts/mmz-arm.sh` does the capture once for every arm, so no tool
needs a quiet flag. It writes each arm's output to `target/check/<gate>.log`,
and the log stays there after a pass. A cache hit doesn't print anything (`on_hit: ""`
in `.mmz/config.yaml`). The arm counts itself as run or cached from the
`skipped` or `ran` mmz writes to the file `MMZ_REPORT` names. Outside #just.check(), #just.mmz("<gate>") streams its
output. #just.test("<filter>") runs outside mmz, so a filtered run never
records a pass for the full suite.

Every rule reads the `gates-meta` scope, which includes every justfile, so a
justfile edit re-runs every arm. `outdatty::check` reads every file
`.config/outdatty.yaml` lists. #just.meta-script-scope() fails naming a listed
file that its scopes miss.

== Inputs a plain glob can't name

#just.crap-check() reads `target/coverage/lcov.info`, which git ignores. mmz drops
ignored paths from a scope, so the `lcov` scope sets `gitignore: false`. With
the lcov in its inputs, the arm re-runs when #just.cover() writes a new one.

#just.lint-secrets() scans the tree, but no gate-tagged rule may name
`truth/tasks/**`. `mt flip` writes the task file it gates, so such an input
would make the rule stale in the middle of the flip. So the `secrets-tree`
scope is `**` with `exclude: ["truth/tasks"]`. The scan still reads the task
files. A change to one doesn't re-run the arm.

#just.docs-vale-tasks() selects `truth/tasks/**` and carries no `gate` tag. It
is the one arm whose inputs include a task file. #just.check() runs it, and a
task body with a shouted word or an em dash fails the build. The close
predicate, `mmz --is-fresh --tag gate`, skips the arm. `mt flip` stamps the
status into the file the arm reads. A tagged record would go stale on the close
that writes it, and the close could never pass. A close doesn't check the task
prose. The arm's last #just.check() does.

== Arms must not read what another arm writes

The arms share one working tree. If an arm reads files another arm writes, it
has to stay out of them or take the writer's lock.

- `www/generate.sh` deletes and recreates `www/generated/` on every run. Every
  reader and writer of that directory holds the `target/www-generated.lock`
  flock for its whole run. The readers regenerate through
  `mmz just docs::gen` inside that lock. The generator runs once per tree.
- #just.docs-pages-fixture() copies `www/` into a temporary replica. It
  skips `public/` and `.tola/`, which the #just.docs-check() arm writes, and
  regenerates `www/generated/` inside the lock.
- #just.lint-secrets() derives a gitleaks allowlist from the paths git
  ignores. The scan skips `www/generated/` and `www/.tola/`. gitleaks
  still stats the skipped directory itself, and the scan holds the lock too.
  `.just/scripts/check-secrets.sh` has the measurements.
- mmz hashes a rule's inputs before it runs the rule, and aborts with exit 8
  if a file vanishes during the hash. So `www/.gitignore` ignores the
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
`--tests`. A build with other features writes to another target directory.
#just.build-mt() does that with `target/mt-dev`.

#just.meta-gate-drift() fails naming file and line on any build of
`mindtape-cli` that a #just.check() arm reaches without `--tests` or a flag
that writes elsewhere.

== One scope per script

`.mmz/config.yaml` sets one scope per file under `.just/scripts/`. A single
glob over the directory would re-run every gate when one script changed. A
list covers only the scripts someone added to it. #just.meta-script-scope()
fails on a script outside every scope, on a rule that reaches a script its
inputs miss, and on a script that's a source of no `.config/outdatty.yaml`
group. Put a new script in the group of the doc that
restates it, or in the source-only group when no doc does.
`.just/scripts/script-scope-check.sh` explains each arm.

= The formatter path lists

#just.fmt() and #just.fmt-check() read one list per formatter from the
`Justfile`, so a path you add there reaches both. #just.meta-gate-drift()
fails when `TOML_FMT_PATHS` misses a TOML file git lists.

#for (name, value) in fact("gates").at("assignments") [
  - #raw(name): #value.split(" ").map(raw).join(", ")
]

just's `--fmt` flag alone formats the root `Justfile` and skips every `mod`
file (verified on just 1.46). So each file `JUST_FMT_PATHS` matches gets its
own `--justfile <file>` pass, `--fmt` in #just.fmt() and `--dump` diffed
against the file in #just.fmt-check(), and each needs its own
`set unstable := true`. #just.fmt-check() lists every module that differs and
fails on any one, not only the last: a bare `for` over the files keeps just the
last body's exit status. #just.meta-fmt-fixture() runs that check over two
drifted modules and a clean one that sorts last.

= Pre-commit assertions

`.githooks/pre-commit` runs `mmz --is-fresh --tag gate`, then
`.githooks/local-paths`, `typstyle --check truth`, `vale-prose` over
`truth/tasks` and `mt check`. It refuses the commit when any of them fails.

`.githooks/local-paths` refuses a staged file that names your `$HOME`, the
checkout or the primary checkout. Such a path arrives in a pasted error or
log, and it is local only to your machine, so no gate in CI can catch it.
Write `<repo>`, `<worktree>` or `~` in its place. `.githooks/commit-msg` runs
the same scan over the message.

`MINDTAPE_GATE_SCOPE=<reason>` skips the first assertion only. The other four
still run, and `.githooks/commit-msg` refuses the commit unless its message
has a matching `Gate-Scope: <reason>` trailer. So `git log` shows which
commits skipped the gate. CONTRIBUTING.md says when you may use it.

A commit that stages only files under `truth/tasks/` skips the first assertion
too, with no trailer. Every gate-tagged rule skips those files, and the Vale
assertion reads their prose. So the freshness assertion could only fail where
the gates never ran, such as a fresh #just.fork() filing its task.
