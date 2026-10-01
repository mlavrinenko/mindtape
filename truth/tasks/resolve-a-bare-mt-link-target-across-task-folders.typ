#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve a bare mt link target across task folders",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent links across folders by stem, the way the help text says it can],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one target lookup instead of a stem that works only for siblings],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 1),
)

== Problem

+ In a project with `cli/spectrogram-parity.typ` and `docs/a.typ`, run `mt link a depends-on spectrogram-parity`.
+ 🔴 It refuses: `` `depends-on` edge target `spectrogram-parity.typ` names no task; did you mean: spectrogram-parity? `` — the suggestion is the very name it just refused.
+ Only `mt link a depends-on ../cli/spectrogram-parity.typ` lands. `settle_targets` in `crates/mindtape-cli/src/link_plan.rs` turns a bare stem into `<stem>.typ`, which resolves next to the linking task only, while `mt link --help` promises "Target tasks, by stem (`other`)".


== Environment

- `mt 0.3.0` at `dba4e21b`, Linux.


== TO-BE

- 🟢 A bare stem (no `/`) that names no sibling but exactly one loaded task is written as the relative path from the linking task to it: `../cli/spectrogram-parity.typ`.
- 🟢 A sibling with that stem still wins, written as `<stem>.typ`, so same-folder links stay byte-identical.
- 🟢 A stem two folders share is refused, naming every candidate path, instead of guessing.
- 🟢 A target with a `/` keeps its literal, relative-to-the-linking-task meaning.


== Changes

- `crates/mindtape-cli/src/link_plan_tests.rs`: red tests first — a cross-folder bare stem stages `../cli/x.typ`; a shared stem is refused naming both paths.
- `crates/mindtape-cli/src/link_plan.rs`: `settle_targets` looks a bare stem up among the loaded tasks when no sibling carries it; `target_problem` names the candidates of an ambiguous stem.
- `crates/mindtape-cli/src/util.rs`: hosts `relative_target`, moved from `crates/mindtape-cli/src/rm_gitless.rs` so both callers share it.
- `crates/mindtape-cli/src/link_cmd.rs`: the `targets` help says how a bare stem resolves.


== Out of scope

- Stem lookup in the Typst DSL or `mt check`: a written edge stays a path relative to its file, so the files read the same without `mt`.
- Stem uniqueness across folders as a check rule: a separate policy question.


== Dogfood

Not applicable: this repo keeps every task in `truth/tasks/`, so all its links are siblings.

