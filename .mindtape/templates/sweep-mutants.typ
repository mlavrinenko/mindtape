#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: slot("title", prompt: "pin <function> in <path>"),
  kind: "test",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises with what the unpinned line decides",
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: cargo-mutants ran the suite, it did not guess",
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how far the mutant sits from a seam",
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
  ),
  estimate: slot("estimate", kind: "text", prompt: "(files: N)"),
  assignee: "agent",
  tags: slot("tags", kind: "list", prompt: "swept, mutants"),
)

== Problem

+ Run `cargo mutants` over the file below.
+ 🔴 #raw(slot("function", prompt: "the mutated function")) in #raw(slot("path", prompt: "repo-relative path of the mutated file")) survived a mutation at line #slot("line", kind: "text", prompt: "line number of the mutant"): the suite still passed after #raw(slot("mutation", prompt: "cargo-mutants' own wording for the mutation, so it greps against missed.txt")).

The line is covered, so something executes it, but nothing asserts on what it
decides. `just ask-refactoring` filed this, and the sweep re-reads the signal
every run, so the task is worth only as much as that measurement still is.

#slot(
  "scope",
  kind: "text",
  prompt: "one sentence naming what that sweep covered",
)

== Goals

#goals(
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
)

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
