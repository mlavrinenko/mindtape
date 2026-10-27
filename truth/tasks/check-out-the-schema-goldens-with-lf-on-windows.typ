#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check out the schema goldens with LF on Windows",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the Windows job runs its whole suite again instead of stopping on a line-ending diff],
      docs-match-code: helps[windows.md says the Windows checkout is CRLF and how a byte-compared file opts out],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 27),
)

== Problem

+ Dispatch `ci.yml` on `main` at `364f88ad`, as the 0.3.0 release preflight did (run 40567353299).
+ 🔴 The `Test (Windows)` job fails `at::schema::tests::at_schema_matches_golden` with `schema/at.schema.json drifted from its structs` and `identical for 141 line(s); committed has 141, generated has 141`.
+ Every line matches, so only the line endings differ. The runner's git checks text out with `core.autocrlf=true`, the golden reads back with CRLF, and the generated schema has LF. `schema_matches_golden` compares `schema/config.schema.json` the same way.
+ nextest stops on the first failure, so 3853 of the 3899 tests never ran.


== Environment

- `main` at `364f88ad`, on the `windows-latest` GitHub runner.
- On Linux, a clone made with `git clone -c core.autocrlf=true` fails the same two tests.


== TO-BE

- 🟢 Both schema goldens pass on Windows, because git checks `schema/*.json` out with LF there too.
- 🟢 One red Windows run lists every failing test, not just the first.


== Changes

- `.gitattributes`: check `schema/*.json` out with `eol=lf`, so the goldens hold the bytes the generator writes on every platform. The red test is the two golden tests in a `core.autocrlf=true` clone.
- `.github/workflows/ci.yml`: run the Windows job's nextest with `--no-fail-fast`.
- `docs/src/contributing/windows.typ`: say that the Windows checkout is CRLF and that a byte-compared file needs `eol=lf`.


== Out of scope

- A repo-wide `* text eol=lf`. It would also hide how `mt` reads CRLF task files that Windows users write.
- The Unix-only merge-driver tests, whose bash scripts also break in a CRLF checkout. They never run on Windows.


== Dogfood

This repo's Windows CI job turns green on the schema goldens, and a red run lists every failure at once.

