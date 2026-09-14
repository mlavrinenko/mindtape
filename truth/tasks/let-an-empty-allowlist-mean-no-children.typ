#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let an empty allowlist mean no children",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("substrate", "gating"),
  estimate: (files: 4),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

Forbidding subsections under a section is naturally `action = "allowlist"` with the section as `scope` and an empty `allowed`, but artifact-gate refuses it: `Rule::allow` returns `RuleError::EmptyPath` when `allowed` is empty (artifact-gate-0.1.0/src/lib.rs:208). This projects config works around it with `action = "forbid"` over a wildcard path pair, which is correct and reads as a trick rather than as the rule it is.

== Scope

Decide whether an empty `allowed` should mean "no children permitted" in artifact-gate. If so, release it and replace the wildcard-pair workaround:

```toml
# today
action = "forbid"
path = [{ match = ".*" }, { match = ".*" }]

# proposed
action = "allowlist"
scope = ["Section"]
allowed = []
```

The empty-scope case stays an error either way, since a scope matching nothing is a typo rather than an intent.

Dogfood: the replacement lands in this repo's `.mindtape/config.toml`.
