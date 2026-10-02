<!-- Generated from docs/src/contributing/example-projects.typ by `just docs md`. Do not edit; edit the source. -->

# The gated example projects

Why `just check-examples` validates every example project, and how it makes each one validate against this checkout’s own prelude.

## Why the examples are gated

Other tools run `mt` against the example projects. `learn-piano` feeds the docs generator and the skill’s triage evals, and `minimal` feeds the skill’s trap evals. A config format change can break an example without any error, so `just check-examples` runs `mt check` on each one.

A new example joins the recipe’s loop and the `examples` scope in `.mmz/config.yaml`. mmz re-runs the gate only when a file in that scope changes, so an example outside it stops being checked.

## Why each example validates against this checkout

A task file imports `@local/mindtape`. Typst resolves that import from one directory per machine, which `mt install` fills from whichever checkout ran it last. A worktree’s example would then validate against another checkout’s prelude, and pass while testing nothing, in exactly the case a prelude change needs it.

So each example’s config points the import at this checkout’s `lib/` with a `[[typst.local-packages]]` entry. The recipe also sets `MINDTAPE_DATA_HOME` to an empty value, which `mt` reads as no directory, so no machine-wide package resolves at all. That also lets the gate run on a machine that never ran `mt install`, such as CI. `XDG_DATA_HOME` would not work, because `dirs` reads it on Linux only.
