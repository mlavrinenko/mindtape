<!-- Generated from docs/src/contributing/example-projects.typ by `just docs md`. Do not edit; edit the source. -->

# The gated example projects

Deep dive behind CONTRIBUTING.md’s gate table: why every example project is validated against itself, and what makes that validation mean something.

## Why the example projects are gated

`just check-examples` validates every example project against itself, so a fixture another tool runs `mt` against cannot rot: `learn-piano` feeds the docs generator and the skill’s triage evals, `minimal` its trap evals. Gating them earns the right to depend on them — the eval fixture before these sat in untracked scratch and silently outlived two config migrations (the retired `[ls] default-filter`, the `depends-on:` → `links:` move), so no eval could have run against it, and nothing said so.

“Against itself” is enforced, not assumed. A task file imports `@local/mindtape`, which Typst resolves from one directory per machine — the one `mt install` fills, pointing at whichever checkout installed last — so a worktree fixture validated against the MAIN tree’s prelude and passed while gating nothing, exactly the case a prelude change needs it for. Each fixture now pins the import to this checkout’s `lib/` with a `[[typst.local-packages]]` entry, and the recipe runs with `MINDTAPE_DATA_HOME` set empty — “no such directory”, so no global package resolves at all — which also makes the gate runnable on a machine that never ran `mt install`, i.e. every CI run. That variable rather than `XDG_DATA_HOME`, the gate’s first reach: `dirs` reads the environment on Unix only, so the XDG spelling isolated the gate on Linux, left macOS reading the machine’s packages, and meant nothing on Windows.
