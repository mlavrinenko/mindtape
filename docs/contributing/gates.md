<!-- Generated from docs/src/contributing/gates.typ by `just docs-md`. Do not edit; edit the source. -->

# Quality gates

Behind CONTRIBUTING.md’s Quality Gates: how `just check` runs, what keeps the gates honest, when a commit may skip a fresh run.

## How check runs

`just check` runs its gates in parallel; CONTRIBUTING.md’s table lists them, derived from the recipe’s dependencies. Each arm is `just mmz <subgate>`, so a no-op re-run skips arms with unchanged inputs and `mt flip` into a terminal status asserts them all fresh via `mmz --is-fresh --tag gate` without running one. Scopes and the `gate` tag live in `.mmz/config.yaml`, where the rule name is `just <recipe>` — the double hop makes mmz the cache identity. `just test <filter>` stays outside mmz (plain `cargo nextest run` with `ARGS`), so no cached record reflects a filtered subset. No `chronic`: a hit prints nothing (`on_hit: ""` in `.mmz/config.yaml` — a skipped arm used to restate “mmz: skipped `<command>` (inputs unchanged)” on every green re-run, noise an exit code already carries), a miss streams live, and a failure exits through `parallel`.

The table’s Command column quotes a gate’s body only while that body is one line of at most 160 characters that does more than name a `.sh` script; anything else renders the `[doc(...)]` string instead (`gate_command` in `www/gates.jq`). A comment inside the body is a body line, so explaining a flag there demotes the row with no doc source touched. Put the comment above the recipe.

`just check-crap` reads `target/coverage/lcov.info` — git-ignored, under no source scope — so it gets an `lcov` scope declared `gitignore: false`, the opt-out that stops a scope over ignored paths resolving empty under the manifest-wide filter. Absent and present are then different input states: the arm skips loudly with no lcov, goes stale the moment `just cover` writes one, and never caches “I could not measure” as a pass outliving the artifact. `just check-secrets` takes the whole tree — a credential lands anywhere — and `.mmz/config.yaml` has no negative glob; `truth/tasks/**` is also the one directory a gate-tagged rule may not name, since `mt flip` writes the task file it gates and would flip the close gate stale mid-command. So its inputs are an enumeration standing in for a wildcard, false-fresh the moment an unlisted top-level directory arrives — so the arm judges the enumeration itself, with `git ls-files` set-differenced against mmz’s own expansion, failing by name. Files it cannot name are still scanned, unmemoized: the trade `typstyle --check truth` takes.

### Arms must not read what another arm writes

The arms share one tree, so an arm reading a tree another arm writes must bound itself out of that tree or serialise against its writer. Four instances:

- `www/generate.sh` deletes and recreates `www/generated/` on every run, so every reader and writer of it serialises on a `target/www-generated.lock` flock, held for the reader’s WHOLE run and not across its own regenerate alone, or the `rm -rf` still lands mid-read as a vanished `crate-map.json`.
- `just site-pages-fixture-check` copies `www/` into a replica entry by entry to skip `public/` and `.tola/`, written by the parallel `just docs check` arm, and regenerates `www/generated/` inside the lock rather than reading it.
- The gitleaks walk in `just check-secrets` is bounded out of `www/generated/` and `www/.tola/` by the unscoped `paths` allowlist in `.gitleaks.toml`, whose patterns take the `^x(/|$)` form, not `^x/`: measured on gitleaks 8.30 under `--log-level debug`, `^gen/` never matches the directory `gen` itself, so the walk ENTERS it and skips each child, while `^gen(/|$)` skips it and never descends. The bound stops there: under strace the `lstat` of the bounded directory lands one syscall BEFORE the allowlist is consulted, so a `rm -rf` between `readdir` and that `lstat` still logs `could not get info` at ERR on an exit-0 run — 8 runs in 20, reproduced. So arm two takes the flock above across its scan, and arm four holds the bound to BOTH `.gitignore` files — covered, or `# gitleaks-bound: exempt` — in `.just/scripts/_gitleaks-bound-check.sh`.
- mmz is stricter under the same rule: it resolves inputs before running the command, so a transient file inside a hashed tree that vanishes mid-hash aborts the run outright (exit 8, “a file mmz needed could not be read or written”). Hence `www/.gitignore` ignoring `.goutputstream-*`, the temp `rsvg-convert` leaves for milliseconds when GIO saves the favicon rasters atomically. That file is the single declaration of what the docs build owns: mmz filters its input walk by it, `.gitleaks.toml` restates it by hand for arm four.

## Why the script scopes are enumerated

`.mmz/config.yaml` names one scope per file under `.just/scripts/`. A single `.just/scripts/**` glob in `gates-meta`, the scope every rule shares, used to re-run every rule — clippy and the full test suite and the docs build included — for an edit to one recipe body no other gate opens. The cost: that directory is recipe bodies by construction, so under the glob a body extracted from the Justfile was covered by arriving, while an enumeration covers only what someone listed and the rule running an unlisted script serves a stale hit against a body it never hashed.

`just script-scope-check` pays for that. It set-differences the files on disk against the literal paths the scopes name, then walks the call graph out of each rule’s recipe body to demand that a rule reaching a script declare a scope covering it — a second arm, because a script listed in a scope no rule reads passes the first while the stale hit survives. A wildcard never counts as naming a script. `outdatty.yaml` enumerates the same directory and lost the same property: a script never in its `dev-docs` group couples to no dependent and is reported by nothing. A third arm demands a DECISION, not membership (`_nav-diff.sh` and the machine-install scripts are outside it on purpose, no doc restating their logic), so a script is a source or carries an exemption naming the reason, and a new one is neither. Three rules keep a directory-wide glob, their subject being the directory, not one body in it: `just check-file-size` (linecop scans every file under it), `just outdatty-check` (nearly every script is a `dev-docs` source) and `just script-scope-check` itself, which must notice a script ARRIVING.

That call graph (`.just/scripts/_script-graph.sh`) is transitive and treats a recipe and a script as one kind of node, so a script invoking a script and a script shelling out to a nested `just` are the same hop — `script-nav-diff`, the two gitleaks-bound scripts on `just check-secrets` and check-crap.sh’s `just crap` were each hand-wired first, which is what a missing derivation looks like from the outside. Reading one word too many fails a CORRECT manifest and the obvious repair is to add the input rather than fix the reader, so what counts as a call is argued line by line in that script, and `.just/scripts/_script-graph-control.sh` pins every one of those shapes on a synthetic tree before arm two runs: from the tracked tree, the right answer and a walk that stopped following edges print the same sentence.

### What the split did and did not buy

Measured on the maintainer’s machine, hot `target/`, against the twenty-two rules the manifest carried then (the twenty-one `just check` arms of the day plus `just cover`); arms have been added since, so this is the measurement, not a current reading. Each figure is one `just check` run after the edit named, with `dev-docs` re-recorded as such an edit requires:

- No-op re-run: 21 of 21 skipped, 0.2s wall / 0.5s CPU. The floor.
- One-script edit: 4 of 21 re-run — the rule whose body it is, plus the three directory-wide ones — 0.8s wall / 1.5s CPU. Before the split, all of them.
- Justfile-only edit: 21 of 21, 42s wall / 105s CPU. Unchanged by this work, and roughly 50x the one-script case.

Every rule names `gates-meta` and the Justfile is one file, so hashing part of it needs mmz’s command-driven input hashes — filed separately and deliberately not adopted here, where a wrong probe would make a gate lie; this measurement is what that decision waits on. What the averages hide: a one-script edit costs the edited arm’s own runtime, not the fan-out, so editing `.just/scripts/check-doc-recipes.sh` still takes 38s, because that arm alone takes 38s — and it sets the gate’s parallel wall-clock floor. Fan-out is what the scopes control; a slow arm is its own problem.

## Why the example projects are gated

`just check-examples` validates every example project against itself, so a fixture another tool runs `mt` against cannot rot: `learn-piano` feeds the docs generator and the skill’s triage evals, `minimal` its trap evals. Gating them earns the right to depend on them — the eval fixture before these sat in untracked scratch and silently outlived two config migrations (the retired `[ls] default-filter`, the `depends-on:` → `links:` move), so no eval could have run against it, and nothing said so.

“Against itself” is enforced, not assumed. A task file imports `@local/mindtape`, which Typst resolves from one directory per machine — the one `mt install` fills, pointing at whichever checkout installed last — so a worktree fixture validated against the MAIN tree’s prelude and passed while gating nothing, exactly the case a prelude change needs it for. Each fixture now pins the import to this checkout’s `lib/` with a `[[typst.local-packages]]` entry, and the recipe runs with `MINDTAPE_DATA_HOME` set empty — “no such directory”, so no global package resolves at all — which also makes the gate runnable on a machine that never ran `mt install`, i.e. every CI run. That variable rather than `XDG_DATA_HOME`, the gate’s first reach: `dirs` reads the environment on Unix only, so the XDG spelling isolated the gate on Linux, left macOS reading the machine’s packages, and meant nothing on Windows.

## Why the module files are formatted one by one

A bare `--fmt` pass formats the root `Justfile` only and silently skips every `mod` file. Verified, not assumed: on just 1.46, against a synthetic module, `--fmt` left the mis-formatted module byte-identical and `--fmt --check` reported only the root file’s diff. It had already cost something — the docs module sat outside `just fmt` and `just fmt-check` from the day it was split out (`truth/tasks/split-the-justfile-into-modules.typ`) and had drifted by one blank line before anything looked (`truth/tasks/gate-formatting-of-just-module-files.typ`). So each module file gets its own `--justfile <file> --fmt` pass over one `JUST_FMT_PATHS` list shared by both recipes; a module file that moves without its entry moving drops back out of both and reopens that hole.

## What the pre-commit hook asserts

The hook `nix develop` installs asserts the gate is fresh via `mmz --is-fresh --tag gate`, checks task files, and refuses a stale commit.

## The escape hatch

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the freshness assertion; `typstyle --check truth` and `mt check` still run. Not a silent skip either — `.githooks/commit-msg` refuses the commit unless the message carries a matching `Gate-Scope: <reason>` trailer, so the record of what was not verified travels in `git log` with the commit that did not verify it.

## When it is legitimate

One case: a delegated worktree whose full aggregate runs at integration — the gate deferred, not skipped, to the run that provably covers the merged result, which is the only run that proves it (a branch passing its gate in isolation can still break the integrated tree). Dodging a red gate is never legitimate: if the gate fails, the commit is wrong, and a trailer saying so does not make it right.
