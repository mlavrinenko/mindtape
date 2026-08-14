#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "extract the long gate recipe bodies into scripts",
  priority: ice(
    impact: 8,
    confidence: 0.75,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("tooling",),
  estimate: (files: 15),
  links: depends-on(
    "move-the-just-module-and-scripts-into-a-dot-just-folder.typ",
  ),
  status: done(
    2026,
    8,
    14,
  )[six bodies out, Justfile 949 to 763; gate\_command suppresses bare script invocations so CONTRIBUTING.md is byte-identical; each gate proven to still fail on the violation it exists to catch. Size-gate rationale turned out void — see restore-size-gate-coverage-over-the-dot-just-scripts],
)

== Summary

Every gate recipe body over five lines moves to its own script under
`.just/scripts`, where linecop can see it.

== Why, beyond tidiness

linecop detects language by extension. `Justfile` has none, so at 893 lines it
is the largest un-capped source file in the repo — and it is the file that
defines the size gate. `contrib/ask-refactoring.sh` was capped for exactly this
reason once it reached 750 lines; the Justfile is the same failure, still open.
A `.sh` file lands under the existing `Shell: 300` cap on arrival.

== In scope

The gate arms whose bodies are shebang scripts:

- `check-doc-coverage`
- `check-doc-facts`
- `check-doc-recipes`
- `fmt-fixture-check`
- `check-crap`
- `docs::md-check` (in the module file)

== Out of scope

Bodies already at or under five lines stay inline: `check-dry`, `count-tests`,
`file-sizes`, `install-hooks`, `uninstall-lib`. Moving a two-line pipeline into
a file costs a hop and buys nothing.

== The doc-pipeline consequence, which needs a decision

`www/gates.jq` derives the gate table's Command column from each recipe's
`.body`, using it when the body is a single non-shebang line under 160
characters and falling back to the `[doc]` string otherwise. Extraction turns
these multi-line shebang bodies into one-liners like `bash
.just/scripts/check-doc-facts.sh`, which would newly *qualify* and replace the
doc string with a script path.

A script path is less informative than the doc string in that column. Teach
`gate_command` to return `null` for a body that is a bare script invocation, so
the fallback keeps applying and CONTRIBUTING.md renders unchanged. A rendered
diff in that table is the signal that this was missed.

== Scope

- One script per recipe, named for it, executable, with the `set -eo pipefail`
  contract each body already declares. Never `set -u`.
- `check-crap` calls `just _lcov-stale-reason`; that seam is the sibling
  extraction task's subject. Keep the call as-is here — do not inline it, and do
  not race the other task for the same file.
- The comments currently above each recipe stay where they are in this task.
  Their relocation is its own task, and doing both at once makes the diff
  unreviewable.
- `outdatty.yaml`: add the new scripts as `dev-docs` sources, per that group's
  own argument about bodies leaving the Justfile. Re-confirm its seven
  dependents with `just outdatty-update`.
- Confirm the gate table renders byte-identical, and that `just check` is green
  on the merged tree, not only in isolation.
