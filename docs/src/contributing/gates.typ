// docs/src/contributing/gates.typ — source for docs/contributing/gates.md.
// Rendered by docs/generate-md.sh via typlite, same as every other
// docs/src/*.typ source; see docs/src/lib.typ for the render path's
// constraints.
//
// Nothing here is derived, and nothing here should be: a memoization
// contract, a fixture that rotted and a formatter behaviour verified against
// one just version are process and measurement, not values any of
// www/generate-facts.sh's artifacts carries. The gate list itself is derived,
// and lives once, in CONTRIBUTING.md's table.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/gates.md",
)) <mindtape-md>

#doc-title[Quality gates]

Behind CONTRIBUTING.md's Quality Gates: how #just.check() runs, what keeps the
gates honest, when a commit may skip a fresh run.

= How check runs

#just.check() runs its gates in parallel; CONTRIBUTING.md's table lists them,
derived from the recipe's dependencies. Each arm is #just.mmz("<subgate>"), so a
no-op re-run skips arms with unchanged inputs and `mt flip` into a terminal
status asserts them all fresh via `mmz --is-fresh --tag gate` without running
one. Scopes and the `gate` tag live in `.mmz/config.yaml`, where the rule name is
#raw("just <recipe>") — the double hop makes mmz the cache identity.
#just.test("<filter>") stays outside mmz (plain `cargo nextest run` with `ARGS`),
so no cached record reflects a filtered subset. No `chronic`: a hit prints
nothing (`on_hit: ""` in `.mmz/config.yaml`, because a skip line on every green
re-run is noise the exit code already carries), a miss streams live, and a
failure exits through `parallel`.

The table's Command column quotes a gate's body only while that body is one
line of at most 160 characters that does more than name a `.sh` script;
anything else renders the `[doc(...)]` string instead (`gate_command` in
`www/gates.jq`). A comment inside the body is a body line, so explaining a flag
there demotes the row with no doc source touched. Put the comment above the
recipe.

#just.check-crap() reads `target/coverage/lcov.info` — git-ignored, under no
source scope — so it gets an `lcov` scope declared `gitignore: false`, the
opt-out that stops a scope over ignored paths resolving empty under the
manifest-wide filter. Absent and present are then different input states: the arm
skips loudly with no lcov, goes stale the moment #just.cover() writes one, and
never caches "I could not measure" as a pass outliving the artifact.
#just.check-secrets() takes the whole tree — a credential lands anywhere — and
`.mmz/config.yaml` has no negative glob; `truth/tasks/**` is also the one
directory a gate-tagged rule may not name, since `mt flip` writes the task file
it gates and would flip the close gate stale mid-command. So its inputs are an
enumeration standing in for a wildcard, false-fresh the moment an unlisted
top-level directory arrives — so the arm judges the enumeration itself, with
`git ls-files` set-differenced against mmz's own expansion, failing by name.
Files it cannot name are still scanned, unmemoized: the trade
`typstyle --check truth` takes.

== Arms must not read what another arm writes

The arms share one tree, so an arm reading a tree another arm writes must bound
itself out of that tree or serialise against its writer. Four instances:

- `www/generate.sh` deletes and recreates `www/generated/` on every run, so every
  reader and writer of it serialises on a `target/www-generated.lock` flock, held
  for the reader's whole run and not across its own regenerate alone, or the
  `rm -rf` still lands mid-read as a vanished `crate-map.json`.
- #just.site-pages-fixture-check() copies `www/` into a replica entry by entry to
  skip `public/` and `.tola/`, written by the parallel #just.docs-check() arm,
  and regenerates `www/generated/` inside the lock rather than reading it.
- The gitleaks walk in #just.check-secrets() is bounded out of `www/generated/`
  and `www/.tola/` by an unscoped `paths` allowlist `check-secrets.sh` derives
  from what git ignores, whose patterns take the `^x(/|$)` form, not `^x/`: measured on gitleaks 8.30 under
  `--log-level debug`, `^gen/` never matches the directory `gen` itself, so the
  walk enters it and skips each child, while `^gen(/|$)` skips it and never
  descends. The bound stops there: under strace the `lstat` of the bounded
  directory lands one syscall before the allowlist is consulted, so a `rm -rf`
  between `readdir` and that `lstat` still logs `could not get info` at level `ERR` on an
  exit-0 run — 8 runs in 20, reproduced. So arm two takes the flock above across
  both the derivation and its scan.
- mmz is stricter under the same rule: it resolves inputs before running the
  command, so a transient file inside a hashed tree that vanishes mid-hash aborts
  the run outright (exit 8, "a file mmz needed could not be read or written").
  Hence `www/.gitignore` ignoring `.goutputstream-*`, the temp `rsvg-convert`
  leaves for milliseconds when GIO saves the favicon rasters atomically. That
  file is the single declaration of what the docs build owns: mmz filters its
  input walk by it, and the gitleaks walk bound is derived from it.

== One mt binary, one feature set

`target/debug/mt` is read by the #just.test() arm (`CARGO_BIN_EXE_mt`) and
rebuilt by every arm that needs an `mt`. It is one path for two builds:
`mindtape-cli`'s dev-dependencies turn on `mindtape/test-support` and
`swhid-mint/test-support`, so the build under `cargo test --no-run` is a
different unit, hence a different `target/debug/deps/mt-<hash>`, from a plain
`cargo build -p mindtape-cli`. Cargo uplifts by hard link and swaps the link
whenever it points at another inode, fresh unit or not (on a warm tree
`cargo build` after `cargo nextest run --no-run` gives `mt` a new inode with no
compile; a repeat gives none). So a sequential pre-build before the fan-out
cannot help: each later build of the other feature set swaps `mt` back.

Measured on a tree stale in `crates/`, watching the inode: in two runs a plain
`cargo build` (`check-doc-coverage.sh` or `www/generate.sh`) swapped `mt` back
after the test build had linked its own, once at the instant nextest began
running tests and once 18 s before. A swap is a delete then a link: a window of
about 85 ms in which a spawn fails with `No such file or directory`. Every arm
that builds `mt` for the gate runs `cargo build -p mindtape-cli --tests`,
the invocation that resolves the same unit as the test arm, so every uplift
after the first is a no-op. That leaves one relink per stale run, during the
test arm's own build, and none once it runs. `check-examples` builds once and runs
`./target/debug/mt`, since `cargo run` takes no `--tests`. A build with another
feature set belongs in another target directory: #just.build-mt() writes
`target/mt-dev`. #just.check-rrule() builds the `rrule` set into the shared
`target/debug` on purpose and stays safe by not being a #just.check() arm: CI
runs it on its own, its `cargo run` after its own nextest, so no test is in flight.

#just.check-gate-drift() holds the rule: it fails naming file and line on any
`cargo build`/`cargo run` of `mindtape-cli` that a #just.check() arm reaches
(dependencies, a nested `just`, any script run) without `--tests` or a flag that
writes elsewhere (`--release`, `--target-dir`). The reach is derived, never listed,
so #just.check-rrule() is outside it. It also checks `TOML_FMT_PATHS` against git.

= Why the script scopes are enumerated

`.mmz/config.yaml` names one scope per file under `.just/scripts/`. A single
`.just/scripts/**` glob in `gates-meta`, the scope every rule shares, would
re-run every rule — clippy, the full test suite and the docs build included —
for an edit to one recipe body no other gate opens. The cost: a glob covers a
new script the moment it lands, while an enumeration covers only what someone
listed, and the rule running an unlisted script serves a stale hit against a
body it never hashed.

#just.script-scope-check() pays for that. It set-differences the files on disk
against the literal paths the scopes name, then walks the call graph out of each
rule's recipe body to demand that a rule reaching a script declare a scope
covering it — a second arm, because a script listed in a scope no rule reads
passes the first while the stale hit survives. A wildcard never counts as naming
a script.
`outdatty.yaml` enumerates the same directory, with the same gap: a
script never in its `dev-docs` group couples to no dependent and is reported by
nothing. A third arm demands a decision, not membership (`_nav-diff.sh` and the
machine-install scripts are outside it on purpose, no doc restating their logic),
so a script is a source or carries an exemption naming the reason, and a new one
is neither. Four rules keep a directory-wide glob, their subject being the
directory, not one body in it: #just.check-file-size() (linecop scans every file
under it), #just.outdatty-check() (nearly every script is a `dev-docs` source),
#just.check-gate-drift() (it follows #just.check() into any script an arm runs)
and #just.script-scope-check() itself, which must notice a new script.

That call graph (`.just/scripts/_script-graph.sh`) is transitive and treats a
recipe and a script as one kind of node, so a script invoking a script and a
script shelling out to a nested `just` are the same hop. Reading one word too
many fails a correct manifest, and the obvious repair is to add the input
rather than fix the reader, so what counts as
a call is argued line by line in that script, and
`.just/scripts/_script-graph-control.sh` pins every one of those shapes on a
synthetic tree before arm two runs: from the tracked tree, the right answer and a
walk that stopped following edges print the same sentence.

== What the split did and did not buy

Measured on the maintainer's machine, hot `target/`, against the twenty-two rules
the manifest carried then (the twenty-one #just.check() arms of the day plus
#just.cover()); arms have been added since, so this is the measurement, not a
current reading. Each figure is one #just.check() run after the edit named, with
`dev-docs` re-recorded as such an edit requires:

- No-op re-run: 21 of 21 skipped, 0.2s wall / 0.5s CPU. The floor.
- One-script edit: 4 of 21 re-run — the rule whose body it is, plus the three
  directory-wide ones — 0.8s wall / 1.5s CPU. Before the split, all of them.
- Justfile-only edit: 21 of 21, 42s wall / 105s CPU. Unchanged by this work, and
  roughly 50x the one-script case.

Every rule names `gates-meta` and the Justfile is one file, so hashing part of it
needs mmz's command-driven input hashes — filed separately and deliberately not
adopted here, where a wrong probe would make a gate lie; this measurement is what
that decision waits on. What the averages hide: a one-script edit costs the
edited arm's own runtime, not the fan-out, so editing
`.just/scripts/check-doc-recipes.sh` still takes 38s, because that arm alone
takes 38s — and it sets the gate's parallel wall-clock floor. Fan-out is what the
scopes control; a slow arm is its own problem.

= Why the module files are formatted one by one

A bare `--fmt` pass formats the root `Justfile` only and silently skips every
`mod` file. Verified, not assumed: on just 1.46, against a synthetic module,
`--fmt` left the mis-formatted module byte-identical and `--fmt --check` reported
only the root file's diff. It had already cost something — the docs module sat
outside #just.fmt() and #just.fmt-check() from the day it was split out
(`truth/tasks/split-the-justfile-into-modules.typ`) and had drifted by one blank
line before anything looked
(`truth/tasks/gate-formatting-of-just-module-files.typ`). So each module file
gets its own `--justfile <file> --fmt` pass over one `JUST_FMT_PATHS` list shared
by both recipes; a module file that moves without its entry moving drops back out
of both and reopens that hole.

= What the pre-commit hook asserts

The hook `nix develop` installs asserts the gate is fresh via
`mmz --is-fresh --tag gate`, checks task files, and refuses a stale commit.

= The escape hatch

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the freshness assertion;
`typstyle --check truth` and `mt check` still run. Not a silent skip
either — `.githooks/commit-msg` refuses the commit unless the message carries a
matching `Gate-Scope: <reason>` trailer, so the record of what was not verified
travels in `git log` with the commit that did not verify it.

= When it is legitimate

One case: a delegated worktree whose full aggregate runs at integration — the
gate deferred, not skipped, to the run that provably covers the merged result,
which is the only run that proves it (a branch passing its gate in isolation can
still break the integrated tree). Dodging a red gate is never legitimate: if the
gate fails, the commit is wrong, and a trailer saying so does not make it right.
