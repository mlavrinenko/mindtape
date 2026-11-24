#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Read the dry arm's clones from jscpd's duplicates, not its classes",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  links: (
    (
      task: "explain-or-fix-every-clone-the-dry-gate-tolerates.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  assignee: "agent",
  status: done(
    2026,
    11,
    25,
  )[27 tasks filed from the 28 debt reasons, which are 27 clones],
)

== AS-IS

The dry arm reads a jscpd array that 5.4.3 does not write.

+ Explaining every clone the dry gate tolerates, then running the sweep
+ 🔴 #raw("contrib/ask-refactoring/dry.sh"): the dry arm reads jscpd's `classes` array, which 5.4.3 does not emit, so `just advisory ask-refactoring` fails before it files a clone

```
$ jq -r '[.classes[] | select(.isNew | not)] | length' /tmp/dryrep/jscpd-report.json
jq: error (at /tmp/dryrep/jscpd-report.json:49288): Cannot iterate over null (null)

$ jq 'keys' /tmp/dryrep/jscpd-report.json
[
  "duplicates",
  "statistics"
]
```


== TO-BE

- 🟢 `score_dry` reads the report's `duplicates` pairs, groups them by fragment, and keeps the groups a debt pair belongs to.
- 🟢 `just advisory ask-refactoring` files the clones the dry gate tolerates that the reasons file records as debt. The 28 debt reasons become 27 tasks: two fingerprints belong to one clone of three copies.

```
$ just advisory ask-refactoring
dry            27 finding(s)
  dry-crates-mindtape-src-rules-tests-rs-f159389a  [3 copies, gain 135]
  ...
```


== Changes

- `contrib/ask-refactoring/dry.sh`: replace the `.classes` read with the `duplicates` grouping the repeats gate uses, and update the header.
- `docs/src/contributing/signal-sweep.typ`: the dry arm paragraph says what the arm reads.


== Out of scope

- The reasons file and the gate, which `explain-or-fix-every-clone-the-dry-gate-tolerates` lands.
- The 28 debt extractions, which the arm files once it reads the right array.


== Dogfood

`just advisory ask-refactoring` files the 27 tasks the reasons file records as debt, and the baseline shrinks as they close.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[the dry arm files the clones the gate tolerates. The debt the reasons file records becomes work].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

