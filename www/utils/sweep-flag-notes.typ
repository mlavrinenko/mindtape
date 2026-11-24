// Hand-written rationale, one entry per `just advisory ask-refactoring` flag, keyed by
// the flag's long name. The flag list, each flag's help text and each default
// come from www/generated/sweep-flags.json (www/generate-facts.sh, reading
// `just --dump`); this dict contains only what a machine cannot state: why the
// number is its own number rather than one band shared across the sweep's
// arms. An entry must never restate the help text or the default:
// docs/src/contributing/signal-sweep.typ renders both alongside, derived, so a
// copy here would be the drift this file exists to end. `just docs::facts`
// fails naming any sweep-flags.json entry with no matching key here.
//
// The full argument for each band — the distributions, the counts, what each
// edge rejects and why it stops where it does — stays in the arm script that
// applies it (`contrib/ask-refactoring/*.sh`), deliberately: prose written
// away from the constants it explains would fall out of date with them. These
// entries refer to that script.
//
// Two entries below cross-reference other gates (`just cover`, `just crap::check`)
// to explain what those measure that this sweep's own arms do not, so those
// mentions go through `#just.<name>()` too — a hand-typed recipe name is
// exactly the drift docs::recipes exists to catch, and it scans this
// directory. That makes this file a reader of www/generated/ (the generated
// `just` module), which is only safe because the renderers that reach it —
// docs::md and docs::facts — hold the www/generated/ flock across their
// typst queries as well as their own regenerate (see docs/generate-md.sh and
// the docs::facts recipe for the fd-held idiom). If you add a consumer
// that renders this dict outside one of those, it must hold that lock for
// its whole run or read no generated file at all, the way crate-notes.typ
// and dsl-field-notes.typ do.
//
// Refs: truth/tasks/2026-08/derive-the-ask-refactoring-flag-table-from-arg-attributes.typ

#import "../generated/just.typ" as just

#let sweep-flag-notes = (
  "write": [
    The one flag that is not a threshold: it decides whether a finding is
    printed or filed, never which findings there are. Dry is the default
    because the sweep is a proposal: you read one before it becomes a task,
    and the remedy stays the refactorer's call.
  ],
  "baseline": [
    A percentage of each file's own cap, not a line count. The caps differ per
    language and per override, so the one number that means the same thing
    everywhere is how much of its own budget a file uses. It sits above
    linecop's own `--baseline 85` reporting band. The sweep then files a task
    strictly later than the number linecop tells contributors to watch.
  ],
  "cc-floor": [
    It pairs with `--fn-coverage-floor`: the sweep reports only functions at
    or above this cyclomatic complexity and below the coverage floor.
    #just.crap-check() gates complexity separately, at its own threshold
    against a committed baseline. Below the floor, one uncovered line can
    account for most of a low percentage. At or above it, whole branches went
    untested.
  ],
  "fn-coverage-floor": [
    The other half of the pair, measured per function rather than per file. It
    is stricter than #just.cover()'s 88% per-file floor on purpose: a function
    is small enough that one unreached line moves its percentage by tens of
    points. The low-coverage tail then starts further out. The pair has two
    edges rather than one shared number because a function must be both complex
    enough and undertested enough.
  ],
)
