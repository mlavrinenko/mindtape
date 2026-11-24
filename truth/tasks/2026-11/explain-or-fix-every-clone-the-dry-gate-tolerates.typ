#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Explain or fix every clone the dry gate tolerates",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("stateless", "tooling", "gating"),
  assignee: "agent",
  status: proposed(2026, 11, 24),
  links: related(
    "gate-short-repeats-on-jscpds-native-duplicates-and-baseline.typ",
  )[the gate reads the baseline this task explains],
)

== AS-IS

The dry gate tolerates 1,294 clones and explains none of them.

+ Run `jq '.fingerprints | length' .config/jscpd-baseline.json`.
+ 🔴 It prints 1294: every clone `just lint dry` lets through, and none carries a reason.
+ Re-mint the baseline with `just lint dry-baseline`.
+ 🔴 jscpd 5.4.3 adds 83 clones the fork missed and drops 161 it found. A re-mint moves the set unexplained. 921 are test-to-test.
+ Read `.config/jscpd-baseline.json`.
+ 🔴 It maps a fingerprint to `1` and nothing else, so a reader cannot tell a coincidence from a clone nobody got to.
+ Read the close rule in `docs/src/contributing/signal-sweep.typ`.
+ 🔴 It says a clone closes by extraction or by a `jscpd:ignore` marker whose comment says why. `just lint dry-baseline` re-mints the baseline with no reason at all.

== TO-BE

- 🟢 Every fingerprint in `.config/jscpd-baseline.json` has a reason in `.config/jscpd-baseline-reasons.json`, or leaves the file because the clone got a fix.
- 🟢 `just lint dry` fails on a baseline fingerprint the reasons file does not explain, and a re-mint cannot add one silently.
- 🟢 Each fixed clone drops its fingerprint, and the reasons file drops its reason.

```
$ just lint dry
dry: 1294 clones tolerated, every one explained

$ just lint dry
dry: 1 baseline entry has no reason: 0007733b7e5896df
hint: fix the clone, or add its reason to .config/jscpd-baseline-reasons.json
```

== Changes

- `.config/jscpd-baseline.json`: the clones that stay after the fixes, minus the ten stale fingerprints.
- `.config/jscpd-baseline-reasons.json`: one reason per fingerprint, and the schema the gate reads.
- `.just/lint.just`: `dry` fails on a baseline fingerprint with no reason.

== Out of scope

- The gate on short repeats, which is its own task.
- The 28 short repeats the repeats arm reports, which the test-helper dedupe tasks fix.
- The clones the gate already refuses, which never reach the baseline.

== Dogfood

This repo's `just lint dry` fails on a baseline entry with no reason, and the baseline shrinks as the dry arm's tasks close.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a tolerated clone carries a reason or leaves the baseline, so the gate cannot hide a clone nobody reviewed].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
