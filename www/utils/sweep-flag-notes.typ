// Hand-written rationale, one entry per `just ask-refactoring` flag, keyed by
// the flag's long name. The flag list, each flag's help text and each default
// come from www/generated/sweep-flags.json (www/generate-facts.sh, reading
// `just --dump`); this dict carries only what a machine cannot state: why the
// number is its own number rather than one band shared across the sweep's
// arms. An entry must never restate the help text or the default:
// docs/src/contributing/signal-sweep.typ renders both alongside, derived, so a
// copy here would be the drift this file exists to end. `just check-doc-facts`
// fails naming any sweep-flags.json entry with no matching key here.
//
// The full argument for each band — the distributions, the counts, what each
// edge rejects and why it stops where it does — stays in the arm script that
// applies it (`contrib/ask-refactoring/*.sh`), deliberately: it is the most
// valuable thing in the arm, and prose that lived away from the constants it
// explains would drift away from them. These entries point there.
//
// Two entries below cross-reference other gates (`just cover`, `just crap`)
// to explain what those measure that this sweep's own arms do not, so those
// mentions go through `#just.<name>()` too — a hand-typed recipe name is
// exactly the drift check-doc-recipes exists to catch, and it scans this
// directory. That makes this file a reader of www/generated/ (the generated
// `just` module), which is only safe because the renderers that reach it —
// docs::md and check-doc-facts — hold the www/generated/ flock across their
// typst queries as well as their own regenerate (see docs/generate-md.sh and
// the check-doc-facts recipe for the fd-held idiom). If you add a consumer
// that renders this dict outside one of those, it must hold that lock for
// its whole run or read no generated file at all, the way crate-notes.typ
// and dsl-field-notes.typ do.
//
// Refs: truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-attributes.typ

#import "../generated/just.typ" as just

#let sweep-flag-notes = (
  "write": [
    The one flag that is not a threshold: it decides whether a finding is
    printed or filed, never which findings there are. Dry is the default
    because the sweep is a proposal — the remedy is the refactorer's call, and
    a proposal you have to read before it becomes a task is how that stays
    true.
  ],
  "baseline": [
    A percentage of each file's own cap, not a line count: the caps differ per
    language and per override, so the one number that means the same thing
    everywhere is how much of a file's own budget is spent. Sits above
    linecop's own `--baseline 85` reporting band, so the sweep files a task
    strictly later than the number a contributor is told to watch.
  ],
  "cc-floor": [
    Half of a conjunction with `--fn-coverage-floor`, and the half that is not
    a complexity judgment — #just.crap() gates complexity, at its own
    threshold against a committed baseline. This floor is what makes the
    coverage number mean something: below it, a low percentage says one line
    is unhit; at or above it, it says decisions are unexercised.
  ],
  "fn-coverage-floor": [
    The other half, per function rather than per file, and stricter than
    #just.cover()'s 88% per-file floor on purpose: a function is small enough
    that one unreached line moves its percentage by tens of points, so the tail
    starts further out. Two edges rather than one shared number is the point of
    the pair — a function has to be both complex enough and undertested enough.
  ],
  "repeat-gain": [
    Tokens an extraction removes, net of the helper and its calls, so one bar
    weighs copies against size: a big block clears it with two copies, a
    one-liner only once it is everywhere. A count or a size alone would file
    the common short call and miss the pasted setup. The number is set so a
    batch stays small enough to empty before the next sweep.
  ],
  "dry-gain": [
    The same measure as `--repeat-gain`, over the clones the jscpd baseline
    tolerates rather than the ones under its floor. Its own flag because the
    two populations differ: lowering one bar should not flood the other arm.
  ],
)
