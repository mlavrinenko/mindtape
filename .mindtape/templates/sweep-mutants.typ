#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "pin <function> in <path>",
    example: "Pin compare_due in crates/mindtape/src/sort.rs",
  ),
  kind: "test",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises with what the unpinned line decides",
      example: 5,
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: cargo-mutants ran the suite, it did not guess",
      example: 0.9,
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how far the mutant sits from a seam",
      example: 6,
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
    example: 2,
  ),
  assignee: "agent",
  tags: slot(
    "tags",
    kind: "list",
    prompt: "swept, mutants",
    default: ("stateless",),
    example: ("swept", "mutants", "stateless"),
  ),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: helps[a line that decides something gets a test that fails when it breaks],
      docs-match-code: neutral,
    ),
  ),
)

== AS-IS

+ Run `cargo mutants` over the file below.
+ 🔴 #raw(slot("function", prompt: "the mutated function", example: "compare_due")) in #raw(slot("path", prompt: "repo-relative path of the mutated file", example: "crates/mindtape/src/sort.rs")) survived a mutation at line #slot("line", kind: "text", prompt: "line number of the mutant", example: "88"): the suite still passed after #raw(slot("mutation", prompt: "cargo-mutants' own wording for the mutation, so it greps against missed.txt", example: "replace compare_due -> Ordering with Ordering::Equal")).

The line is covered, so something executes it, but nothing asserts on what it
decides. `just ask-refactoring` filed this, and the sweep re-reads the signal
every run, so the task is worth only as much as that measurement still is.

#slot(
  "scope",
  kind: "text",
  prompt: "one sentence naming what that sweep covered",
  example: "That sweep ran 40 mutants across 1 file(s) (crates/mindtape/src/sort.rs) and left 1 unpinned.",
)

== TO-BE

- 🟢 `just mutants::sweep -f` on this file reports the mutant as caught, or
  `.config/mutants.toml` declares it uninteresting with a reason.

== Changes

- `crates/**/*_tests.rs`: write the assertion that kills the mutant, and assert
  on the behavior the mutated line decides, not on the literal it was replaced
  with. The assertion has to be one a reader would want anyway.
- `.config/mutants.toml`: declare the mutant uninteresting with a reason next to
  the entry instead. A written reason is the only thing that suppresses this
  finding on the next sweep; there is no ignore list anywhere else,
  deliberately.

== Out of scope

- A test that pins the replaced constant and nothing else: it moves the
  survivor without buying a regression net.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
