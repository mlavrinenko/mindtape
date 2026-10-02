// docs/src/contributing/example-projects.typ — source for
// docs/contributing/example-projects.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// Nothing here is derived: `check-examples`, the `[[typst.local-packages]]`
// pin and the `MINDTAPE_DATA_HOME` trick are hand-written decisions in the
// Justfile and each example's config. It split off gates.typ, where it sat
// behind CONTRIBUTING.md's gate table; that table still lists
// #just.check-examples() and CONTRIBUTING.md still points here.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/example-projects.md",
)) <mindtape-md>

#doc-title[The gated example projects]

Deep dive behind CONTRIBUTING.md's gate table: why every example project is
validated against itself, and what makes that validation mean something.

= Why the example projects are gated

#just.check-examples() validates every example project against itself, so a
fixture another tool runs `mt` against cannot rot: `learn-piano` feeds the docs
generator and the skill's triage evals, `minimal` its trap evals. An ungated
fixture breaks silently when the config format changes.

"Against itself" is enforced, not assumed. A task file imports `@local/mindtape`,
which Typst resolves from one directory per machine — the one `mt install` fills,
pointing at whichever checkout installed last. A worktree's fixture would then
validate against another checkout's prelude and pass while gating nothing, in
exactly the case a prelude change needs it for. So each fixture pins the import
to this checkout's `lib/` with a `[[typst.local-packages]]` entry, and the recipe runs
with `MINDTAPE_DATA_HOME` set empty — "no such directory", so no global package
resolves at all — which also makes the gate runnable on a machine that never ran
`mt install`, i.e. every CI run. Setting `XDG_DATA_HOME` instead would isolate
the gate on Linux only, because `dirs` reads it there alone. macOS would read
the machine's packages, and Windows would ignore it.
