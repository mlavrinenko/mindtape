// The names a task body writes. A task imports this module whole, so the
// helper and each goal value are in scope as written.

#import "goal-dsl.typ": Helps, Hinders, Neutral
#import "goals.typ": goals

#let answer-fast = goals.answer-fast
#let avoid-lock-in = goals.avoid-lock-in
