#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Gate short repeats on jscpd's native duplicates and baseline",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("stateless", "tooling", "gating"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

The repeats arm reports 28 short repeats and gates none of them.

+ Read `contrib/ask-refactoring/repeats.sh`.
+ 🔴 It is an advisory arm: it files a task per snippet and gates nothing, so a 15-token line pasted 20 times lands without a refusal.
+ Run it: `jscpd --no-tips -f rust -k 15 -l 1 -r json -o target/rep crates`, then `score_repeats` with `REPEAT_GAIN=300`.
+ 🔴 It reports 28 findings, all real test-setup repetitions, and the gate that should stop the next one does not exist.
+ Read `nix/dev/shell.nix`.
+ 🔴 jscpd comes from qahq's fork at `v5.4.1-nix.2`, which the arm needs for `classes`. Upstream 5.4.3 has `duplicates` and `--baseline`, and the fork outlives its reason.

== TO-BE

- 🟢 `just lint repeats` gates short repeats at 15 tokens through jscpd's native `duplicates` and `--baseline`, with no fork and no hand-rolled clone-class engine.
- 🟢 The 28 findings get their fixes first, and the gate starts green.
- 🟢 The repeats arm leaves `just ask-refactoring`, and the dry gate keeps its 50-token floor for renamed and similar clones.
- 🟢 jscpd is upstream 5.4.3, not qahq's fork.

```
$ just lint repeats
repeats: no new short repeat
```

== Changes

- `.just/scripts/repeats-gate.sh`: the gate body. It groups jscpd's `duplicates` pairs by fragment, counts the unique copies by `name:start:end`, scores each group `gain = (copies - 1) * tokens - 5 * copies - 10`, drops the arm's noise, folds overlapping groups, and keeps `gain >= 100`. A kept group fails when any pair is new. A tolerated group needs a reason keyed by the fragment's SHA-1. `--mint` rewrites the baseline and lists each group with no reason.
- `.just/lint.just`: the `repeats` and `repeats-baseline` recipes. The dry gate keeps `-k 50`. The repeats gate runs at `-k 15` with exact tokens. The Justfile adds the gate to `_check-fast`, and the mmz manifest its scopes and inputs.
- `.config/jscpd-repeats-baseline.json`: jscpd's native baseline at 15 tokens, minted after the fixes, with the reasons sidecar beside it holding one reason per tolerated group.
- `contrib/ask-refactoring.sh`: the arm and `--repeat-gain` leave the sweep, with the template, the `repeats` tag, the docs and the fixtures.
- `crates/mindtape/src/test_support/fs.rs`: the nine real repeats the gate found after the 28 fixes, extracted into `write_ancestor_defaults`, `write_project_config`, `git`, `wip_task`, `head_dir`, `task_with`, `annotated`, `visible` and the `from_extra` delegation.

== Alternatives

- Keep the repeats arm as the advisory sweep: it re-reports the same snippets every sweep and gates nothing, so the growth the decision exists to stop continues.
- Gate at jscpd's 50-token floor: the 28 findings are below it, so the gate would start green but miss every one of them.
- Put the repeats gate on `.config/jscpd-baseline.json`. A 15-token fingerprint in the 50-token file would make the dry gate tolerate a clone it never measured.
- Read jscpd's `classes`: 5.4.3 has none, and the fork's groups by token count rather than fragment text. A class then names copies of a different snippet.
- Keep qahq's fork: it carries `classes`, but upstream 5.4.3 carries `duplicates` and `--baseline`, so the fork would outlive its reason.

== Out of scope

- The 28 fixes, which the test-helper dedupe tasks land first.
- The jscpd 5.4.3 pin in `nix/dev/shell.nix` and `flake.lock`, which the qahq agent's phase 2 owns. The gate reads `duplicates`, present in 5.4.1 and 5.4.3.
- The clones at 50 tokens and up, which the dry gate refuses.

== Dogfood

This repo's `just lint repeats` refuses a new 15-token snippet pasted twice, and the repeats arm files nothing.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a short snippet pasted again fails the gate instead of waiting for the next sweep].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
