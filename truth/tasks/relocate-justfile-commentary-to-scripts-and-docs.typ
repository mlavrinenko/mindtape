#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "relocate justfile commentary to scripts and docs",
  priority: ice(
    impact: 7,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 6,
  tags: ("tooling",),
  estimate: (files: 15),
  links: depends-on(
    "extract-the-long-gate-recipe-bodies-into-scripts.typ",
    "extract-the-coverage-and-release-recipe-bodies-into-scripts.typ",
  ),
  status: done(
    2026,
    8,
    15,
  )[Justfile 753 to 509 lines, comments only — just --dump byte-identical apart from absolute paths. Every relocated block mapped, and audited by six-word shingle diff against the destination tree; three real losses found that way and put back],
)

== Summary

Keep only execution knowledge in the Justfile. Implementation quirks move into
the script that has them; long rationale moves to `docs/src` and is referred to,
not restated.

== The sorting rule

Three destinations, decided by who needs the sentence:

/ Justfile: what a caller needs before running the recipe — what it does, when
  to reach for it, what it will refuse to do. The `[doc]` string plus at most a
  few lines.
/ The script: why the code inside is written the way it is. `jscpd
  --exit-code 1` needing its value spelled out; `ln -sfn` into an existing
  directory placing the link inside it; the `--min` / `--threshold` pairing
  making the table list only failures; the two-invocation split in `crap`.
/ `docs/src`: the arguments. Why `cover` cleans the profraw pile and what
  happened when it did not; why there is no advisory CRAP band; why there is no
  CI mutation job; why `check-crap` skips loudly instead of failing.

The `crap` recipe alone carries roughly fifty lines of measured argument. That
is a docs page, and `docs/src/contributing/coverage-crap.typ` already exists to
hold it.

== The hard constraint

No comment content may be deleted. Every sentence either stays, moves, or is
replaced by a pointer to where it moved. This project has lost argued prose to a
formatter once already and to a well-meant trim once; both were caught late.
Produce a mapping of every relocated block to its destination and check it
before committing, the way the last milestone's comment audit did.

Judgement calls that go the other way: if a sentence is genuinely redundant with
a `[doc]` string that now says the same thing, it may go — but say so in the
report, per block, rather than dropping it silently.

== Scope

- Do this after both extraction tasks. A comment cannot move into a script that
  does not exist yet.
- `docs/src/*.typ` sources, never the generated Markdown. `just docs::md`
  regenerates `CONTRIBUTING.md` and the deep dives.
- Prose added to `docs/src` must route recipe names through `#just.<name>()`;
  a hand-typed mention fails `check-doc-recipes`. That gate also scans
  `www/utils/`.
- `dev-docs` is busted by definition here. Review all seven dependents properly
  — this is the task where "review the dependents" is the actual work, not a
  formality — then `just outdatty-update`.
- Expect the Justfile to shrink hard. That is the point, and it is what finally
  makes its size defensible without a linecop entry it cannot have.
