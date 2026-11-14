#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "fail a half-wired gate arm at the wiring it is missing",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 10),
  links: related("relativize-the-crap-baseline-s-committed-file-paths.typ")[
    the gate arm whose wiring produced these measurements
  ],
  status: done(
    2026,
    8,
    17,
  )[both probes read true, arm three caught \_nav-diff.sh on its first run],
)

== Summary

Adding one `just check` arm means six coupled edits: the recipe and its `check:`
dependency, a `.mmz` `script-` scope and rule, a `www/utils/gate-notes.typ`
entry, an `outdatty.yaml` `dev-docs` source, and the regenerated Markdown. We probed each miss by hand against the `crap-baseline-check` arm rather than
assuming it. Most of the wiring already answers well. A stale `dev-docs` group
reports the changed source, every dependent to review and the confirm command. A
missing prose entry prints `undocumented gates (no www/utils/gate-notes.typ
entry): - crap-baseline-check`. An unscoped script gets reported by
`script-scope-check` along with the `script-<stem>` scope to add. Two cases do
not, and the fix for both is a check that reports the missing step, not a
checklist in CONTRIBUTING.md asking a reader to remember it.

== Defect: the membership mismatch reads as a dependency that does not exist

`[group("gate")]` present, `check:` dependency absent, measured:

```
gate group and check dependencies disagree:
  only in [group("gate")]: crap-baseline-check
  only in check dependencies: generate-facts.sh: refusing to install malformed gates.json
```

The second line is not a recipe name at all. `www/gates.jq` ends its message with
`halt_error(1)`, which emits the string with no trailing newline, so the next
stderr line from `www/generate-facts.sh` lands on the end of it. The side that is
truly empty renders as an empty `join`, so nothing sits between the colon and the
glued text. The one line a reader would act on is the line that is fiction.

== Hole: nothing couples a new gate script to dev-docs

`outdatty.yaml`'s `dev-docs` group lists `.just/scripts/*.sh` one by one, and
outdatty reacts only to files it lists. A gate script never added to that list is
coupled to no dependent and reported by nothing. The `script-scope-check` first
arm restores that arriving-is-enough property for `.mmz` scopes, but this group
has no equivalent. Same failure shape, one file over.

== Scope

- end `www/gates.jq`'s message with a newline and render an empty side as
  `(none)`, so neither side can ever read as a member of the other
- set-difference `.just/scripts/*.sh` against `outdatty.yaml`'s `dev-docs`
  sources, failing with the line to add. `script-scope-check` already walks that
  directory for the same class of question and is the natural home, as an arm
  rather than a new gate
- out of scope: a `just new-gate <name>` scaffold. Generation cannot be
  forgotten, which beats any check, but it would have to write six files in four
  formats and would rot against every one of them. The two checks above make a
  half-wired arm fail by itself, which is the property we want
- out of scope, explicitly: a CONTRIBUTING.md checklist. A wiring step that a
  contributor can forget gets a check that reports it

== Resolution

Arm three demands a decision, not membership, and the scope above got that
wrong. Written as a plain set-difference it failed on its first run against
`_nav-diff.sh`, a script left out of `dev-docs` on purpose, with the reason
recorded in `outdatty.yaml`. No doc restates its logic, so listing it would ask
for a review of seven dependents that no change to it can rot. Demanding
membership would have forced exactly the coupling this repo declined. So a script
is a source or an exemption with a written reason, and a new one is neither. The
arriving-is-enough property lands on the file nobody decided about, which is the
one that matters. An exemption that contradicts a source line, or that reports a
deleted file, fails as well. We probed all four cases by hand.

The `www/gates.jq` half re-runs the original probe clean: the empty side reads
`(none)` and `generate-facts.sh`'s refusal lands on its own line instead of on
the end of it.

The measured split figures in `docs/src/contributing/gates.typ` read as the
measurement they are rather than a current reading, because arms landed after
that measurement. Re-typing the numbers without re-measuring would have
been the drift this repo files tasks about.
