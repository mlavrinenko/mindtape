// How this repo builds Mindtape. Kept apart from goals.typ, which says what
// Mindtape is for. Every open task rates each goal here with a body marker
// too, checked by `check-goal-over(...)` from truth/lib.typ.
//
// A task names a goal by the kebab of its infinitive, which goal-calls.typ
// binds. The registry keys carry those names. goal-calls.typ also binds the
// old `x-goals` keys as aliases, so a task filed before the rename evaluates.

#import "@local/mindtape:0.5.0": maintainer
#import "@local/mindtape:0.5.0": goal as goal-of

// This set's audience, fixed once: a goal value in process-goals.typ serves the
// people who build this repo, and a verdict written for its readers is a
// wrong-audience breach.
#let goal(to: none, from: none, title: none, acid-test: none) = goal-of(
  maintainer,
  to: to,
  from: from,
  title: title,
  acid-test: acid-test,
)

#let goals = (
  keep-changes-safe: goal(
    to: "keep changes safe",
    from: "keeping changes safe",
    title: "Changes stay safe",
    acid-test: [A change that breaks documented behavior fails `just check` before it can be committed, and every file fits its size cap.],
  ),
  match-docs-to-code: goal(
    to: "match docs to code",
    from: "matching docs to code",
    title: "Docs match the code",
    acid-test: [Every doc claim about the CLI, the config or the task DSL is generated from its source, or fails a gate once the two drift.],
  ),
)
