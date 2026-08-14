#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive the ask refactoring flag table from arg attributes",
  priority: ice(
    impact: 7,
    confidence: 0.55,
    ease: 2,
  ),
  difficulty: 8,
  tags: ("tooling",),
  estimate: (files: 14),
  links: depends-on("validate-recipe-arguments-with-the-arg-attribute.typ"),
  status: proposed(2026, 8, 14),
)

== Summary

Declare the sweep's flags as recipe parameters with `arg` attributes, and derive
their doc table from the just dump — the way the gate table already is.

== Why this one is worth the size

`ask-refactoring` has a real flag interface: `--write`, `--baseline`,
`--coverage-floor`, `--cc-floor`, `--fn-coverage-floor`, each with its own
default and its own reason for having a separate threshold rather than one band
shared across arms. All of it lives in a comment paragraph above the recipe.
`just --list` shows none of it, and `just ask-refactoring --help` does not exist.

The last milestone moved gate membership, row order, gate commands and the
formatter path lists out of hand-written prose and into the dump. This is the
same surface, still hand-written, and the last one of its size.

== Verified: the metadata reaches the dump

```
[arg('COVERAGE_FLOOR', long='coverage-floor', help='per-file line-coverage floor')]
ask-refactoring COVERAGE_FLOOR='88':
```

`just --dump --dump-format json` carries it in full:

```json
{"arg": {"name": "COVERAGE_FLOOR", "long": "coverage-floor",
         "help": "per-file line-coverage floor",
         "pattern": null, "short": null, "value": null}}
```

`parameters[].default` carries `88` alongside, so the defaults column is derived
too, not retyped. And `just --usage ask-refactoring` renders a usage block from
the same data.

== The cost, stated plainly

Flag parsing moves out of `contrib/ask-refactoring.sh` and into the recipe
signature. That script is the sweep's driver and its four arm scripts read the
thresholds it parses. This is the invasive part and the reason the confidence
here is low.

== Scope

- Declare the flags as parameters with `long` and `help`; keep the existing
  defaults, which are load-bearing and separately argued (5 and 80 for the
  undertested arm are a conjunction, hence two edges, deliberately not one
  shared number).
- Forward to `contrib/ask-refactoring.sh` explicitly rather than as a blob, so
  the script's own parsing can shrink to reading what it is given.
- Add a facts artifact for the flags in `www/generate-facts.sh` alongside
  `gates.json`, and have `docs/src/contributing/signal-sweep.typ` read it
  instead of restating the thresholds.
- Extend `check-doc-facts` with a fourth arm, matching the three it already
  runs: every derived flag needs a notes entry, or the gate fails naming it.
- The prose this replaces carries argument, not just values — why each arm has
  its own threshold, why the sweep refuses to run while `mt ls @ask-refactoring`
  is non-empty, why it is deliberately not an `mmz` arm. Derive the table; keep
  the argument as prose beside it.
