#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept a task reference in mt eval",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("mt", "cli"),
  estimate: (files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[the skill's documented `mt eval <task>` works as written],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the skill and the CLI agree on what `<task>` is],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    9,
    27,
  )[stem and handle resolve through util::resolve\_task; a path still works outside a project; unknown stems get a did-you-mean on every command],
)

== Problem

+ `skills/mindtape/SKILL.md` lists `mt eval <task>`, where `<task>` is a stem, a path, or a `#` handle.
+ 🔴 `mt eval refuse-a-dependency-cycle-when-mt-link-writes-it` prints `Error: no such file: refuse-a-dependency-cycle-when-mt-link-writes-it`; `mt eval --help` shows `<FILE>`.


== Environment

- `mt` 0.3.0 and `HEAD`, Linux.


== TO-BE

- 🟢 `mt eval` resolves a stem, path or handle like `flip`, `set` and `link`.
- 🟢 An unknown reference names the nearest stems.


== Changes

- `crates/mindtape-cli/tests/` (red first): `mt eval <stem>` succeeds.
- `crates/mindtape-cli/src/eval.rs`: resolve through the shared task-reference resolver.


== Out of scope

- Evaluating a non-task `.typ`: a path still works.


== Dogfood

Agents here run `mt eval <stem>` as the skill says.

