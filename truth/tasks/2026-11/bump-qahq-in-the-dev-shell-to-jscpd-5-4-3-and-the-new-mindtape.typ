#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Bump qahq in the dev shell to jscpd 5.4.3 and the new mindtape",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("stateless", "tooling"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

The dev shell pins qahq ten commits behind, before jscpd 5.4.3 and the new mindtape.

+ Read `nix/dev/flake.lock`.
+ 🔴 It pins qahq at `97d0174a`, which carries jscpd `v5.4.1-nix.2` from the fork and mindtape `3e83adea`.
+ Run `nix flake update qahq` in `nix/dev` before pushing qahq's main.
+ 🔴 It cannot see qahq's local commits. The pin stays behind.

== TO-BE

- 🟢 The dev shell pins qahq at `b2db993`, which carries jscpd 5.4.3 from upstream and mindtape `8a6a90d4`.
- 🟢 `nix develop` enters with the new qahq, and `just check` passes in it.

```
$ nix develop --command bash -c 'jscpd --version; mt --version'
jscpd 5.4.3
mt 0.6.0
```

== Changes

- `nix/dev/flake.lock`: `nix flake update qahq`, which also moves qahq's own inputs.

== Out of scope

- The qahq bump itself, which qahq's own commits land.

== Dogfood

This repo's dev shell runs the same jscpd and mindtape the gates run.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[the dev shell runs the jscpd and mindtape the gates run].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
