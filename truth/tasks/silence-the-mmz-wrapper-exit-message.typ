#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "silence the mmz wrapper exit message",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (files: 3),
  links: related(
    "adopt-the-script-attribute-for-the-shebang-recipes.typ",
  )[the other modern-just attribute worth adopting],
  status: proposed(2026, 8, 15),
)

== Summary

Every `just check` arm runs through the `mmz` wrapper recipe, so every failing
arm produces two error lines: the real one from the sub-gate, and a second from
`mmz` naming the wrapper and its own line number. `[no-exit-message]` on `mmz`
drops the second and changes nothing else.

== The evidence

From a red `just check` run this session, the two lines the failure produced:

```
line  602: error: Recipe `outdatty-check` failed on line 114 with exit code 1
line 2817: error: Recipe `mmz` failed on line 63 with exit code 1
```

The second line names the wrapper rather than the gate, points at line 63 of the
Justfile (which is `mmz just {{ ARGS }}`, not the failure), and lands 2200 lines
after the line that says what actually broke. `check` is `[parallel]`, so the
wrapper lines are what a reader sees last — the useful line is buried mid-scroll
in the fan-out.

== Scope

- One attribute on `mmz`. `[no-exit-message]` suppresses only the message from
  the recipe that carries it: verified that the inner recipe's own error line
  and the propagated exit code both survive.
- Assert both halves rather than eyeballing the output. Run one deliberately
  failing sub-gate through `just mmz <gate>` and check that the gate's error line
  is still printed and that `$?` is unchanged; then run the full `just check`
  against the same break and confirm the wrapper line is gone from the fan-out.
- Leave `check-crap` alone. It is the one arm `check` invokes directly rather
  than through `mmz`, so it never had a wrapper line to lose.
- Nothing derives recipe attributes into the docs — `www/gates.jq` reads
  `.attributes` only to find `group("gate")` — so no generated artifact should
  move. Confirm that by regenerating rather than by reading this sentence.
- The Justfile is a `dev-docs` outdatty source: reconcile that group's dependents
  and re-record with `just outdatty-update dev-docs`.

== Not in scope

`[no-exit-message]` on the gate recipes themselves. The wrapper's line is noise
because it names the wrapper; a gate's own line names the gate and the exit code,
which is the line a reader wants. Only the wrapper loses its message.
