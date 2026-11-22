// What Mindtape is for. Every open task rates each goal with a body marker,
// checked by `check-goal-over(...)` from truth/lib.typ. How this repo builds
// Mindtape is a separate set, in process-goals.typ.
//
// A task names a goal by the kebab of its infinitive, which goal-calls.typ
// binds. The dict keys stay the names a task rated in `x-goals:` until the
// migration renames them to that kebab.

#import "goal-dsl.typ": adopter
#import "goal-dsl.typ": goal as goal-of

// This set's audience, fixed once: a goal value in goals.typ serves a task's
// reader, and a verdict written for anyone else is a wrong-audience breach.
#let goal(to: none, from: none, title: none, acid-test: none) = goal-of(
  adopter,
  to: to,
  from: from,
  title: title,
  acid-test: acid-test,
)

#let goals = (
  // The goal set for an agnostic engine, decide-the-goal-set-for-an-agnostic-engine.
  readable-tasks: goal(
    to: "read tasks at a glance",
    from: "reading tasks at a glance",
    title: "Tasks read at a glance",
    acid-test: [Anyone opening an open task file can say within 30 seconds what is broken, why it matters, and how it should work instead.],
  ),
  implementable-tasks: goal(
    to: "guide the implementer",
    from: "guiding the implementer",
    title: "Tasks guide the implementer",
    acid-test: [An implementer in a fresh session, given only the task file, lands the change without asking where to start or what done looks like.],
  ),
  enforced-practice: goal(
    to: "run checks at the write",
    from: "running checks at the write",
    title: "A project's rules run at the write",
    acid-test: [`mt add` and `mt flip` refuse what a rule rejects. A git-reading rule reports `skipped`.],
  ),
  config-over-code: goal(
    to: "keep the engine agnostic",
    from: "keeping the engine agnostic",
    title: "A project owns its policy",
    acid-test: [Every status, tag, rule, path and threshold is a config key, and a rule's function is the project's own Typst. Rust owns the vocabulary the schema spells as an enum, the canonical field names and the reserved `markers` input.],
  ),
  cheap-adoption: goal(
    to: "adopt in one line",
    from: "adopting in one line",
    title: "Best practice adopts in one line",
    acid-test: [A repo takes the flagship's practice with one config line, overrides what it disagrees with, and gets later fixes without copying.],
  ),
  typst-over-markdown: goal(
    to: "beat Markdown",
    from: "beating Markdown",
    title: "Typst beats Markdown with frontmatter",
    acid-test: [A field, slot or marker reaches a rule as a value, never by parsing prose. A collector may derive one; a rule reads the value. Prose rules may match markup and headings, but none re-derives a field. A misspelled helper is a compile error.],
  ),
  plain-files: goal(
    to: "keep plain files",
    from: "keeping plain files",
    title: "Plain files, no infrastructure",
    acid-test: [`mt` binary is the whole setup: no daemon, no database, no index, and nothing outside the repo to lose.],
  ),
  self-describing: goal(
    to: "explain itself",
    from: "explaining itself",
    title: "The CLI explains itself",
    acid-test: [A newcomer in a fresh session, given a project and `mt` but no external documentation, can list, file and flip a task, and every refusal names the command that fixes it.],
  ),
)
