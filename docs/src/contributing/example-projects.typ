// docs/src/contributing/example-projects.typ renders
// docs/contributing/example-projects.md through docs/generate-md.sh
// (typlite; docs/src/lib.typ lists the constraints).
//
// The settings this page explains are in .just/scripts/check-examples.sh and
// in each example's `.mindtape/config.toml`.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/example-projects.md",
)) <mindtape-md>

#doc-title[The gated example projects]

Why #just.check-examples() validates every example project, and how it makes
each one validate against this checkout's own prelude.

= Why the examples are gated

Other tools run `mt` against the example projects. `api-service` feeds the
docs generator and the skill's triage evals, and `minimal` feeds the skill's
trap evals. A config format change can break an example without any error,
so #just.check-examples() runs `mt check` on each one.

When you add an example, add it to the list in
`.just/scripts/check-examples.sh` and to the `examples`
scope in `.mmz/config.yaml`. mmz re-runs the gate only when a file in that
scope changes, so an example outside it stops being checked.

= Why each example validates against this checkout

A task file imports `@local/mindtape`. Typst resolves that import from one
directory per machine, which `mt install` fills from whichever checkout ran
it last. A worktree's example would then validate against another checkout's
prelude. It would pass while testing nothing, in exactly the case where a
prelude change needs it.

So each example's config points the import at this checkout's `lib/` with a
`[[typst.local-packages]]` entry. The recipe also sets `MINDTAPE_DATA_HOME`
to an empty value, which `mt` reads as no directory, so no machine-wide
package resolves at all. An example that lost its entry then reads the prelude
built into `mt`, which a test keeps equal to `lib/`, and never another
checkout's. `XDG_DATA_HOME` wouldn't work, because `dirs` reads it on Linux
only.
