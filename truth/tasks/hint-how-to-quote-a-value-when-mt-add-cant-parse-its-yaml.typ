#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hint how to quote a value when mt add can't parse its YAML",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("cli",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a refused `mt add -f -` names the fix, so the filer re-files without guessing at YAML rules],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 25),
)

== Problem

+ Run ```sh printf 'title: Probe\nwhy: `quoted` start\n' | mt add -t tweak -f -```.
+ 🔴 It prints `error: can't parse YAML: '-': found character that cannot start any token at line 2 column 6, while scanning for the next token` and no `hint:` line. A value opening with a backtick, which task prose does all the time, is a reserved YAML indicator.

== Environment

- Commit `2ba8e88d` on Linux. Hit four times in one speconaut session filing tasks through `mt add -f -`.

== TO-BE

- 🟢 A YAML parse error from `mt add -f` carries a `hint:` line: quote a value that starts with a YAML indicator such as a backtick, or write it as a `|-` block.
- 🟢 The `error:` line keeps the source and serde's position.

== Changes

- `crates/mindtape-cli/src/cmd/add/yaml_values_tests.rs`: the red test first, a value opening with a backtick refused with the hint.
- `crates/mindtape-cli/src/cmd/add/yaml_values.rs`: add the hint to the `can't parse YAML` diagnostic.

== Out of scope

- Accepting unquoted backtick values, which YAML reserves.
- The `--set` path, which takes no YAML.

== Dogfood

This repo's agents file findings through `mt add -t finding -f -`, and their slot values often open with a code span.
