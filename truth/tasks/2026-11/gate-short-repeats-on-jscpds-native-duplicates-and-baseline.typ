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
  status: proposed(2026, 11, 24),
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

- 🟢 `just lint dry` gates short repeats at 15 tokens through jscpd's native `duplicates` and `--baseline`, with no fork and no hand-rolled grouping.
- 🟢 The 28 findings get their fixes first, and the gate starts green.
- 🟢 The repeats arm leaves `just ask-refactoring`, and the dry arm owns the baseline.
- 🟢 jscpd is upstream 5.4.3, not qahq's fork.

```
$ just lint dry
dry: no new clone or short repeat
```

== Changes

- `nix/dev/shell.nix`, `nix/dev/flake.lock`: jscpd from upstream 5.4.3, not qahq's fork.
- `.just/lint.just`: `dry` runs jscpd at 15 tokens with `--baseline` and `--new-only`.
- `.config/jscpd-baseline.json`: the baseline at 15 tokens, after the 28 fixes.
- `contrib/ask-refactoring/repeats.sh`: delete it, and drop the arm from `contrib/ask-refactoring.sh`.
- `.mindtape/config.toml`: drop the `repeats` tag.
- `docs/src/contributing/signal-sweep.typ`: drop the arm.

== Alternatives

- Keep the repeats arm as the advisory sweep: it re-reports the same 28 snippets every sweep and gates nothing, so the growth the decision exists to stop continues.
- Gate at jscpd's 50-token floor: the 28 findings are below it, so the gate would start green but miss every one of them.
- Keep qahq's fork: it carries `classes`, but upstream 5.4.3 carries `duplicates` and `--baseline`, so the fork would outlive its reason.

== Out of scope

- The 28 fixes, which the test-helper dedupe tasks land first.
- The dry baseline's reasons, which the dry-baseline task owns.
- The clones at 50 tokens and up, which the gate already refuses.

== Dogfood

This repo's `just lint dry` refuses a new 15-token snippet pasted twice, and the repeats arm files nothing.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a short snippet pasted again fails the gate instead of waiting for the next sweep].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
