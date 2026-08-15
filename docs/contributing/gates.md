<!-- Generated from docs/src/contributing/gates.typ by `just docs-md`. Do not edit; edit the source. -->

# Quality gates

Deep dive behind CONTRIBUTING.md’s Quality Gates section: how `just check` runs, what keeps the gates themselves honest, and the one sanctioned way to commit without a fully fresh run.

## How check runs

`just check` runs every gate recipe in parallel — formatting, clippy, tests, unused deps, file size, drift, the doc gates and the fixture locks. CONTRIBUTING.md’s table lists them all, derived from the recipe’s own dependencies rather than retyped.

Each arm is `just mmz <subgate>`: mmz wraps the sub-gate, so a no-op re-run skips every arm whose declared inputs are unchanged, and `mt flip` into a terminal status asserts them all fresh via `mmz --is-fresh --tag gate` without running any. Per-rule scopes and the `gate` tag live in `.mmz/config.yaml`.

The double hop makes mmz the cache identity — the rule name is `just <recipe>` in `.mmz/config.yaml` — uniform across every sub-gate. `just test <filter>`, a developer query with no mmz around it, bypasses that recipe and stays plain `cargo test` with `ARGS`, so the cached record reflects only the bare, full-suite invocation `just check` runs, never a filtered subset.

There is no `chronic` here: a cache hit prints mmz’s own `on_hit` note to stderr (quiet by construction), a miss streams the underlying recipe’s stdout and stderr live, and a failure propagates its non-zero exit out of `parallel` unchanged.

## Why the example projects are gated

`just check-examples` validates every example project against itself, so a fixture some other tool runs `mt` against can never rot: `learn-piano` feeds the docs generator and the skill’s triage evals, `minimal` feeds the skill’s trap evals.

Gating them is what earns the right to depend on them. The eval fixture that preceded these sat in untracked scratch and silently outlived two config migrations — the retired `[ls] default-filter`, the `depends-on:` → `links:` move — so no eval could have run against it, and nothing said so.

## Why the module files are formatted one by one

A bare `--fmt` pass formats the root `Justfile` only and silently skips every `mod` file. That is verified rather than assumed: on just 1.46, against a synthetic module, `--fmt` left the mis-formatted module byte-identical and `--fmt --check` reported only the root file’s diff.

It had already cost something. The docs module sat outside `just fmt` and `just fmt-check` from the day it was split out (`truth/tasks/split-the-justfile-into-modules.typ`) and had drifted by one blank line before anything looked at it (`truth/tasks/gate-formatting-of-just-module-files.typ`). So each module file gets its own `--justfile <file> --fmt` pass, over one `JUST_FMT_PATHS` list shared by both recipes; a module file that moves without its entry moving drops back out of both and reopens exactly that hole.

## What the pre-commit hook asserts

The hook installed by `nix develop` asserts the gate is fresh (`mmz --is-fresh --tag gate`) and checks task files. A commit whose gate run is stale is refused.

## The escape hatch

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the freshness assertion. Everything else still runs: `typstyle --check truth/tasks` and `mt check` are unaffected.

It is not a silent skip. `.githooks/commit-msg` refuses the commit unless the message carries a matching `Gate-Scope: <reason>` trailer, so a scoped commit never reads like a fully gated one in `git log`. The record of what was not verified travels with the commit that did not verify it.

## When it is legitimate

One case: a delegated worktree whose full aggregate runs at integration. The gate is not being skipped, it is being deferred to a run that provably covers the merged result — which is the only run that proves the integration anyway. A branch that passes its gate in isolation can still break the integrated tree.

It is never legitimate to dodge a gate that is actually red. If the gate fails, the commit is wrong, and a trailer saying so does not make it right.
