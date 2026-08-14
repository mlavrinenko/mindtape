#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "validate recipe arguments with the arg attribute",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 5),
  status: proposed(2026, 8, 14),
)

== Summary

Use just 1.45's `arg` patterns so a malformed argument fails at dispatch instead
of deep inside a recipe body.

== Verified on just 1.46.0

The argument name must be a quoted string — a bare identifier fails with
`Attribute 'arg' got 0 arguments but takes 1 argument`, which reads like a
syntax problem and is not.

```
[arg('VERSION', pattern='\d+\.\d+\.\d+', help='semver, no leading v')]
release VERSION:
```

```
$ just release v0.3.0
error: Argument `v0.3.0` passed to recipe `release` parameter `VERSION` does not match pattern '\d+\.\d+\.\d+'
```

== The recipes worth guarding

/ `release VERSION`: today `just release v0.3.0` reaches the body and fails with
  `requested vv0.3.0 but Cargo.toml is 0.3.0` — a confusing message from the one
  recipe that tags and pushes. This is the strongest case.
/ `bench N` and `gen-bench N`: `\d+`. `just bench abc` currently passes `abc`
  straight to `examples/large/generate.sh`.
/ `eject PCT`: a percentage, so a pattern that rejects `120` as well as `abc`.

== Scope

- Patterns and `help` strings only. Converting `ask-refactoring` to declared
  parameters is a separate, larger task that depends on this one.
- `help` strings surface through `just --usage <recipe>`, which is new in 1.46.
  Write them for a reader who has not opened the Justfile.
- The dev shell pins just 1.46.0 while the host has a different version. Verify
  inside the shell — `direnv exec . just ...` — not on the host. The attributes
  reference page documents `min`/`max` and several other keywords that postdate
  1.46 and will not parse here.
- Add a regression check: a recipe whose pattern rejects a known-bad argument,
  asserted the way `fmt-fixture-check` asserts the formatters actually reject
  drift. A validation attribute nobody exercises is a comment.
