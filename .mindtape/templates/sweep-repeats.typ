#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "extract the snippet <path>:<line> repeats <N> times",
    example: "Extract the snippet crates/mindtape-cli/src/cmd/go.rs:34 repeats 7 times",
  ),
  kind: "refactor",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises with the tokens an extraction saves",
      example: 5,
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, short of high: a repeated shape can be a coincidence",
      example: 0.7,
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how many files hold a copy",
      example: 6,
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
    example: 2,
  ),
  estimate: slot(
    "estimate",
    kind: "text",
    prompt: "(src-files: N)",
    example: "(src-files: 8)",
  ),
  assignee: "agent",
  tags: slot("tags", kind: "list", prompt: "swept, repeats", example: (
    "swept",
    "repeats",
  )),
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
      sound-codebase: helps[a snippet pasted everywhere lives in one helper, so
        a fix lands once],
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Run `just ask-refactoring`.
#slot(
  "measured",
  kind: "raw",
  prompt: "one 🔴 line with the copies, files, tokens and gain, then the locations as a nested list",
  example: ```typ
  + 🔴 This snippet repeats 7 times across 7 files, 16 jscpd tokens each. Extracting it saves about 51 tokens net of the helper and its calls.
    - #raw("crates/mindtape-cli/src/cmd/go.rs:34")
  ```,
)

The first copy reads:

#slot(
  "snippet",
  kind: "raw",
  prompt: "the first copy's lines, verbatim, in a rust code block",
  example: ````typ
  ```rust
  let project = util::open()?;
  util::warn_errors(&project.outcome);
  ```
  ````,
)

`just ask-refactoring` filed this from jscpd's pairs at 15 tokens, under the
50-token floor of `just lint::dry`. The arm groups the pairs by hand until
jscpd's own clone classes ship (`canibump.toml`, `jscpd-clone-classes`), so a
location may belong to a neighboring window of the same snippet.

== TO-BE

- 🟢 The snippet lives in one helper, and `just ask-refactoring` no longer
  names it.

== Changes

#slot(
  "remedy",
  kind: "raw",
  prompt: "one bullet naming the first copy's file in backticks: extract a helper, or mark the copies as intended",
  example: ```typ
  - `crates/mindtape-cli/src/cmd/go.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the repetition is a coincidence.
  ```,
)
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other snippets in the same files. Each has its own task, or none.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
