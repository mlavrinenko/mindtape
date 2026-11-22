#import "/truth/prelude/2/lib.typ": *

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
  assignee: "agent",
  tags: slot(
    "tags",
    kind: "list",
    prompt: "swept, repeats",
    default: ("stateless",),
    example: ("swept", "repeats", "stateless"),
  ),
)

== AS-IS

+ Run `just advisory ask-refactoring`.
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

`just advisory ask-refactoring` filed this from jscpd's clone classes at 15 tokens,
under the 50-token floor of `just lint::dry`. A location may belong to a
neighboring window of the same snippet.

== TO-BE

- 🟢 The snippet lives in one helper, and `just advisory ask-refactoring` no longer
  names it.

== Changes

#slot(
  "remedy",
  kind: "raw",
  prompt: "one bullet naming the first copy's file in backticks: extract a helper, or mark the copies as intended",
  example: ```typ
  - `crates/mindtape-cli/src/cmd/go.rs` and every other file the AS-IS lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the repetition is a coincidence.
  ```,
)
- `crates/**/*_tests.rs`: in tests, a fixture or setup function is the helper.
  A test still reads top to bottom when the helper's name says what it sets up.

== Out of scope

- Other snippets in the same files. Each has its own task, or none.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.

#slot(
  "goals",
  kind: "raw",
  required: true,
  prompt: "Under `== Goals`, rate every goal: one `#Helps` with a reason, then `#Neutral` for the rest",
  example: ```typ
  == Goals

  + #Helps(adopter, to: guide-the-implementer)[because one template slot carries the whole goal block].
  + #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
  ```,
)
