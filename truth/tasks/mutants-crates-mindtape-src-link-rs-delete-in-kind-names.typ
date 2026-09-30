#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "pin kind_names in crates/mindtape/src/link.rs",
  kind: "test",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 4,
  estimate: (src-files: 1),
  assignee: "agent",
  tags: ("swept", "mutants"),
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
      sound-codebase: helps[a line that decides something gets a test that fails
        when it breaks],
      docs-match-code: neutral,
    ),
  ),
  status: done(
    2026,
    9,
    30,
  )[pinned by the mutation-survivor sweep: \`just mutants\` reports no survivor for this mutation],
)

== Problem

+ Run `cargo mutants` over the file below.
+ 🔴 #raw("kind_names") in #raw("crates/mindtape/src/link.rs") survived a mutation at line 244: the suite still passed after #raw("delete ! in kind_names").

The line is covered, so something executes it, but nothing asserts on what it
decides. `just ask-refactoring` filed this, and the sweep re-reads the signal
every run, so the task is worth only as much as that measurement still is.

That sweep ran 178 mutants across 47 file(s) (crates/mindtape-cli/src/add.rs, crates/mindtape-cli/src/add\_values.rs, crates/mindtape-cli/src/check\_commit.rs, crates/mindtape-cli/src/check\_range.rs, crates/mindtape-cli/src/cli\_dump.rs, crates/mindtape-cli/src/completion.rs, and 41 more) and left 137 unpinned, with 5 timing out — a timeout is a non-verdict, not a pass.

== TO-BE

- 🟢 `just mutants -f` on this file reports the mutant as caught, or
  `.cargo/mutants.toml` declares it uninteresting with a reason.

== Changes

- Pick the remedy, then do it: write the assertion that kills the mutant, or
  declare it uninteresting in `.cargo/mutants.toml` with a reason. The
  assertion has to be one a reader would want anyway.
- If you assert, assert on the behaviour the mutated line decides, not on the
  literal it was replaced with.
- If you exclude, write the reason next to the entry in `.cargo/mutants.toml`.
  A written reason is the only thing that suppresses this finding on the next
  sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- A test that pins the replaced constant and nothing else: it moves the
  survivor without buying a regression net.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
