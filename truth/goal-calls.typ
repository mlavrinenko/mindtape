// Every name a task body writes. A task imports this module whole
// (`#import "/truth/goal-calls.typ": *`), so the DSL, both goal sets and each
// goal value are in scope as written:
//
//   #Helps(adopter, to: read-tasks-at-a-glance)[why]
//   #Hinders(maintainer, from: keep-changes-safe)[why]
//   #Neutral(guide-the-implementer, beat-markdown)
//
// The bindings below are the post-migration kebab names. The registry keys stay
// the names a task rated in `x-goals:` until the migration renames them, and
// this file is where the two spellings meet.

#import "goal-dsl.typ": *
#import "goals.typ": goals as product-goals
#import "process-goals.typ": goals as process-goals

// What Mindtape is for.
#let read-tasks-at-a-glance = product-goals.readable-tasks
#let guide-the-implementer = product-goals.implementable-tasks
#let run-checks-at-the-write = product-goals.enforced-practice
#let keep-the-engine-agnostic = product-goals.config-over-code
#let adopt-in-one-line = product-goals.cheap-adoption
#let beat-markdown = product-goals.typst-over-markdown
#let keep-plain-files = product-goals.plain-files
#let explain-itself = product-goals.self-describing

// How this repo builds Mindtape.
#let keep-changes-safe = process-goals.sound-codebase
#let match-docs-to-code = process-goals.docs-match-code
