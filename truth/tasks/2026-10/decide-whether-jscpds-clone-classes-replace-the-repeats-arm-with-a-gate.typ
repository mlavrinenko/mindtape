#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Decide whether jscpd's clone classes replace the repeats arm with a gate",
  kind: "decision",
  priority: ice(
    impact: 3,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "repeats"),
  estimate: (src-files: 3),
  assignee: "max",
  status: done(
    2026,
    11,
    24,
  )[The repeats arm gives way to a gate on jscpd's native `duplicates` and `--baseline`, with no fork, on upstream 5.4.3. The 28 findings are real, and their fixes land before the gate. The baseline holds only false positives, each with a reason],
  links: related(
    "../2026-11/gate-short-repeats-on-jscpds-native-duplicates-and-baseline.typ",
  )[the gate this decision calls for]
    + related(
      "../2026-11/explain-or-fix-every-clone-the-dry-gate-tolerates.typ",
    )[the baseline reasons this decision requires],
)

== AS-IS

The repeats arm is an advisory because a gate at 15 tokens would fail today's tree.

+ Read the `jscpd-clone-classes` blocker that `.config/canibump.toml` carried before the commit that dropped it.
+ 🔴 Its plan was to drop the repeats arm of `just ask-refactoring` and gate short repeats in `just lint dry` once jscpd could detect clone classes. The pinned qahq fork includes them, and nothing tracks the plan.
+ Run the repeats arm on its own: `jscpd --no-tips -f rust -k 15 -l 1 -r json -o target/rep crates`, then count `classes` with 300 tokens of gain.
+ 🔴 It finds 26 snippets, such as a create-parent-then-write pair pasted 92 times, so a gate at that bar fails today's tree.


== TO-BE

- 🟢 The repeats arm gives way to a gate on jscpd's native `duplicates` and `--baseline`. No fork: upstream 5.4.3 carries both, and `nix/dev/shell.nix` drops qahq's `v5.4.1-nix.2`.
- 🟢 The 28 findings the arm reports are real, not false positives. The AS-IS quotes 26 from the first run, and the arm reports 28. The fixes land before the gate, and the gate starts green.
- 🟢 The baseline is only false positives, each with a reason. A clone that is not a false positive gets a fix, not a baseline entry.
- 🟢 The gate is `feat` task `gate-short-repeats-on-jscpds-native-duplicates-and-baseline`. The fixes are the test-helper dedupe tasks. The baseline's reasons are `dev` task `explain-or-fix-every-clone-the-dry-gate-tolerates`.


== Changes

- `truth/tasks/2026-10/decide-whether-jscpds-clone-classes-replace-the-repeats-arm-with-a-gate.typ`: records this decision, with no code changes.


== Alternatives

- Keep the repeats arm as it is: it re-reports the same 26 snippets every sweep and gates nothing, so the growth the decision exists to stop continues.
- Gate at jscpd's 50-token floor: 26 snippets with 300 tokens of gain fail today's tree. The gate would start red.

== Out of scope

- The dry gate at 50 tokens, which already gates new clones.


== Dogfood

This repo's own short repeats are what the decision is about.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[short repeats stop growing between sweeps instead of waiting for one].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
