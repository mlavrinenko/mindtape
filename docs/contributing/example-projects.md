<!-- Generated from docs/src/contributing/example-projects.typ by `just docs md`. Do not edit; edit the source. -->

# The gated example projects

Deep dive behind CONTRIBUTING.md’s gate table: why every example project is validated against itself, and what makes that validation mean something.

## Why the example projects are gated

`just check-examples` validates every example project against itself, so a fixture another tool runs `mt` against cannot rot: `learn-piano` feeds the docs generator and the skill’s triage evals, `minimal` its trap evals. An ungated fixture breaks silently when the config format changes.

“Against itself” is enforced, not assumed. A task file imports `@local/mindtape`, which Typst resolves from one directory per machine — the one `mt install` fills, pointing at whichever checkout installed last. A worktree’s fixture would then validate against another checkout’s prelude and pass while gating nothing, in exactly the case a prelude change needs it for. So each fixture pins the import to this checkout’s `lib/` with a `[[typst.local-packages]]` entry, and the recipe runs with `MINDTAPE_DATA_HOME` set empty — “no such directory”, so no global package resolves at all — which also makes the gate runnable on a machine that never ran `mt install`, i.e. every CI run. Setting `XDG_DATA_HOME` instead would isolate the gate on Linux only, because `dirs` reads it there alone. macOS would read the machine’s packages, and Windows would ignore it.
