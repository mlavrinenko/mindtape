// Every name a task body writes. A task imports this module whole
// (`#import "/truth/goal-calls.typ": *`), so the DSL, both goal sets and each
// goal value are in scope as written:
//
//   #Helps(adopter, to: read-tasks-at-a-glance)[why]
//   #Hinders(maintainer, from: keep-changes-safe)[why]
//   #Neutral(guide-the-implementer, beat-markdown)
//
// The registry keys and these bindings are the kebab names.

#import "goal-dsl.typ": Helps, Hinders, Neutral, adopter, goal, maintainer
#import "goals.typ": goals as product-goals
#import "process-goals.typ": goals as process-goals

// What Mindtape is for.
#let read-tasks-at-a-glance = product-goals.read-tasks-at-a-glance
#let guide-the-implementer = product-goals.guide-the-implementer
#let run-checks-at-the-write = product-goals.run-checks-at-the-write
#let keep-the-engine-agnostic = product-goals.keep-the-engine-agnostic
#let adopt-in-one-line = product-goals.adopt-in-one-line
#let beat-markdown = product-goals.beat-markdown
#let keep-plain-files = product-goals.keep-plain-files
#let explain-itself = product-goals.explain-itself

// How this repo builds Mindtape.
#let keep-changes-safe = process-goals.keep-changes-safe
#let match-docs-to-code = process-goals.match-docs-to-code
