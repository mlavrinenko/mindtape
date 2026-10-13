#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "extract the clone at <path>:<line> (<N> copies)",
    example: "Extract the clone at crates/mindtape-cli/src/cmd/add/tests_subprocess.rs:40 (3 copies)",
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
  tags: slot("tags", kind: "list", prompt: "swept, dry", example: (
    "swept",
    "dry",
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
      sound-codebase: helps[a clone the gate tolerates lives in one helper, or carries a
        comment saying why it matches, so the baseline shrinks],
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
  + 🔴 This clone appears 3 times across 2 files, 62 jscpd tokens at most. Extracting it saves about 99 tokens net of the helper and its calls.
    - #raw("crates/mindtape-cli/src/cmd/add/tests_subprocess.rs:40")
  ```,
)

The first copy reads:

#slot(
  "snippet",
  kind: "raw",
  prompt: "the first copy's first lines, verbatim, in a rust code block",
  example: ````typ
  ```rust
  let project = util::open()?;
  util::warn_errors(&project.outcome);
  ```
  ````,
)

`just check-dry` fails a new clone, and `.jscpd-baseline.json` lists this one as
older than the gate. The arm joins jscpd's pairs into a family by shared
location, so a location may belong to a neighbouring window.

== TO-BE

- 🟢 The clone lives in one helper, or its copies carry a `jscpd:ignore`
  marker whose comment says why they match, and `just dry-baseline` has
  dropped it from the baseline.

== Changes

#slot(
  "remedy",
  kind: "raw",
  prompt: "one bullet naming the first copy's file in backticks: extract a helper, or mark the copies as intended",
  example: ```typ
  - `crates/mindtape-cli/src/cmd/add/tests_subprocess.rs` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with `// jscpd:ignore-start` and `// jscpd:ignore-end` and a comment saying why the match is a coincidence. Either way, re-mint the baseline with `just dry-baseline` in the same commit.
  ```,
)
- In tests, a fixture or setup function is the helper. A test still reads top
  to bottom when the helper's name says what it sets up.

== Out of scope

- Other clones in the same files. Each has its own task, or none.
- The gate's threshold and modes.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
