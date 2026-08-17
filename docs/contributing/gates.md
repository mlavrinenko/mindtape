<!-- Generated from docs/src/contributing/gates.typ by `just docs-md`. Do not edit; edit the source. -->

# Quality gates

Deep dive behind CONTRIBUTING.md’s Quality Gates section: how `just check` runs, what keeps the gates themselves honest, and the one sanctioned way to commit without a fully fresh run.

## How check runs

`just check` runs every gate recipe in parallel. CONTRIBUTING.md’s table lists them, derived from the recipe’s own dependencies rather than retyped.

Each arm is `just mmz <subgate>`, so a no-op re-run skips every arm whose declared inputs are unchanged, and `mt flip` into a terminal status asserts them all fresh via `mmz --is-fresh --tag gate` without running any. Per-rule scopes and the `gate` tag live in `.mmz/config.yaml`.

The double hop makes mmz the cache identity — the rule name is `just <recipe>` in `.mmz/config.yaml` — uniform across every sub-gate. `just test <filter>` is a developer query with no mmz around it: it stays plain `cargo test` with `ARGS`, so the cached record only ever reflects the bare full-suite run, never a filtered subset.

There is no `chronic` here. A cache hit prints mmz’s own `on_hit` note to stderr, a miss streams the underlying recipe’s stdout and stderr live, and a failure propagates its non-zero exit out of `parallel` unchanged.

`just check-crap` shows what the wrapping needs. It reads `target/coverage/lcov.info`, a path git ignores and no source scope covers, so it gets an `lcov` scope of its own, declared `gitignore: false` — the per-scope opt-out that keeps a scope over ignored paths from resolving empty under the manifest-wide filter (`.mmz/config.yaml` measures, beside the scope, when that key is load-bearing and when it is only a declaration). Absent and present are then different input states: the arm skips loudly while no lcov exists and goes stale the moment `just cover` writes one, instead of caching “I could not measure” as a pass that outlives the artifact appearing.

## Why the script scopes are enumerated

`.mmz/config.yaml` names one scope per file under `.just/scripts/`, path by path. A single `.just/scripts/**` glob inside `gates-meta` — the scope every rule shares — used to re-run every rule, clippy and the full test suite and the docs build included, for an edit to one recipe body no other gate opens.

The enumeration gives up something real: the directory is recipe bodies by construction, so under the glob a body extracted out of the Justfile was covered by arriving. An enumeration only covers what somebody listed, and the rule that runs an unlisted script serves a stale hit against a body it never hashed.

`just script-scope-check` pays for that, so care does not have to. It set-differences the files on disk against the literal paths the scopes name, then reads each rule’s recipe body out of the just dump to require that a rule invoking a script declares a scope covering it — the second arm because a script listed in a scope no rule reads passes the first while the stale hit survives. A wildcard never counts as naming a script, so putting the glob back satisfies nothing.

`outdatty.yaml` enumerates the same directory and lost the same property: a script never added to its `dev-docs` group is coupled to no dependent and reported by nothing. A third arm closes it, demanding a DECISION rather than membership — `_nav-diff.sh` is outside it on purpose, no doc restating its logic — so a script is a source or carries an exemption naming the reason, and a new one is neither.

Three rules keep a directory-wide glob, because their subject is the directory rather than one body in it: `just check-file-size` (linecop scans every file under it), `just outdatty-check` (nearly every script is a `dev-docs` source) and `just script-scope-check` itself, which has to notice a script ARRIVING and so cannot take an enumeration as its input.

### What the split did and did not buy

Measured on the maintainer’s machine with a hot `target/`, against the twenty-two rules the manifest carried then (the twenty-one `just check` arms of the day plus `just cover`); arms have been added since, and the figures are the measurement rather than a current reading. Each is one `just check` run after the edit named, with the `dev-docs` group re-recorded as any such edit requires:

- A no-op re-run skips 21 of 21 arms: 0.2s wall, 0.5s CPU. The floor.
- A one-script edit re-runs 4 of 21 — the rule whose body it is, plus the three directory-wide ones above: 0.8s wall, 1.5s CPU. Before the split it re-ran all of them, which is the Justfile figure below.
- A Justfile-only edit re-runs 21 of 21: 42s wall, 105s CPU. Unchanged by this work, and roughly 50x the one-script case.

So script edits moved from the Justfile figure to the no-op figure, and the Justfile itself moved not at all. Every rule names `gates-meta` and the Justfile is one file, so hashing part of it needs mmz’s command-driven input hashes — filed separately and deliberately not adopted here, where a wrong probe would make a gate lie. This measurement is what that decision waits on.

One caveat the averages hide: a one-script edit now costs the edited arm’s own runtime rather than the fan-out. Editing `.just/scripts/check-doc-recipes.sh` still takes 38s, because that arm alone takes 38s — and it also sets the whole gate’s parallel wall-clock floor. Fan-out is what the scopes control; a slow arm is its own problem.

## Why the example projects are gated

`just check-examples` validates every example project against itself, so a fixture some other tool runs `mt` against cannot rot: `learn-piano` feeds the docs generator and the skill’s triage evals, `minimal` feeds the skill’s trap evals.

Gating them earns the right to depend on them. The eval fixture that preceded these sat in untracked scratch and silently outlived two config migrations — the retired `[ls] default-filter`, the `depends-on:` → `links:` move — so no eval could have run against it, and nothing said so.

“Against itself” is enforced rather than assumed. A task file imports `@local/mindtape`, which Typst resolves from one directory per machine — the one `mt install` fills, pointing at whichever checkout installed last — so a fixture in a worktree validated against the MAIN tree’s prelude and passed while gating nothing, which is exactly the case a prelude change needs it for. Each fixture now pins the import to this checkout’s `lib/` with a `[[typst.local-packages]]` entry, and the recipe runs with `MINDTAPE_DATA_HOME` set empty — “no such directory”, so no global package resolves at all — and a fixture that lost its entry fails here instead of passing against some other tree. That also makes the gate, and the whole aggregate, runnable on a machine that never ran `mt install`, which is every CI run.

The variable rather than `XDG_DATA_HOME`, which is what the gate reached for first: `dirs` reads the environment on Unix only, so the XDG spelling isolated the gate on Linux, left macOS reading the machine’s packages, and meant nothing on Windows. `mt` resolves the local-package directory through its own setting on every platform, so the recipe now says what it means everywhere.

## Why the module files are formatted one by one

A bare `--fmt` pass formats the root `Justfile` only and silently skips every `mod` file. Verified rather than assumed: on just 1.46, against a synthetic module, `--fmt` left the mis-formatted module byte-identical and `--fmt --check` reported only the root file’s diff.

It had already cost something: the docs module sat outside `just fmt` and `just fmt-check` from the day it was split out (`truth/tasks/split-the-justfile-into-modules.typ`) and had drifted by one blank line before anything looked at it (`truth/tasks/gate-formatting-of-just-module-files.typ`). So each module file gets its own `--justfile <file> --fmt` pass over one `JUST_FMT_PATHS` list shared by both recipes. A module file that moves without its entry moving drops back out of both and reopens exactly that hole.

## What the pre-commit hook asserts

The hook installed by `nix develop` asserts the gate is fresh (`mmz --is-fresh --tag gate`) and checks task files. A commit whose gate run is stale is refused.

## The escape hatch

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the freshness assertion. Everything else still runs: `typstyle --check truth/tasks` and `mt check` are unaffected.

It is not a silent skip. `.githooks/commit-msg` refuses the commit unless the message carries a matching `Gate-Scope: <reason>` trailer, so the record of what was not verified travels with the commit that did not verify it, and a scoped commit never reads like a fully gated one in `git log`.

## When it is legitimate

One case: a delegated worktree whose full aggregate runs at integration. The gate is deferred rather than skipped, to a run that provably covers the merged result — the only run that proves the integration anyway, since a branch that passes its gate in isolation can still break the integrated tree.

Dodging a gate that is actually red is never legitimate. If the gate fails, the commit is wrong, and a trailer saying so does not make it right.
