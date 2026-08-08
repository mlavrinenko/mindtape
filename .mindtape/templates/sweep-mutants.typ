#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "pin <function> in <path>"),
  priority: ice(
    impact: slot("impact", kind: "int", prompt: "1-10, rises with what the unpinned line decides"),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: cargo-mutants ran the suite, it did not guess",
    ),
    ease: slot("ease", kind: "int", prompt: "1-10, falls with how far the mutant sits from a seam"),
  ),
  difficulty: slot("difficulty", kind: "int", prompt: "0-9 on the judgment scale"),
  estimate: slot("estimate", kind: "text", prompt: "(files: N)"),
  tags: slot("tags", kind: "list", prompt: "swept, mutants"),
)

== Summary

#raw(slot("function", prompt: "the mutated function")) in #raw(slot("path", prompt: "repo-relative path of the mutated file")) survived a mutation at line #slot("line", kind: "text", prompt: "line number of the mutant"): the suite still passed after #raw(slot("mutation", prompt: "cargo-mutants' own wording for the mutation, so it greps against missed.txt")).

The line is covered — something executes it — but nothing asserts on what it
decides.

`just ask-refactoring` filed this from `cargo mutants`, and nothing recorded
that it did. The sweep re-reads the signal every run, so this task is worth
exactly as much as the measurement behind it still is.

#slot("scope", kind: "text", prompt: "one sentence naming what that sweep covered")

== Scope

- Pick the remedy, then do it. Either write the assertion that kills the mutant,
  or declare it uninteresting in `.cargo/mutants.toml` with a reason. Both are
  correct answers. A test written only to kill a mutant is not: the assertion
  has to be one a reader would want anyway.
- If you assert, assert on the behaviour the mutated line decides, not on the
  literal it was replaced with. A test that pins the constant and nothing else
  moves the survivor without buying a regression net.
- If you exclude, write the reason next to the entry in `.cargo/mutants.toml`. A
  written reason is the only thing that suppresses this finding on the next
  sweep — there is no ignore list anywhere else, deliberately.
- Done when `just mutants -f` on this file reports the mutant as caught.
