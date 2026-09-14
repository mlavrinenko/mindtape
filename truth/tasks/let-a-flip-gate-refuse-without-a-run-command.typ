#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let a flip gate refuse without a run command",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "gating"),
  estimate: (files: 8),
  kind: "feat",
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

A `[[on.flip]]` gate can only refuse a flip through a shell command's exit code, so a hook whose `when` already says everything — refuse starting a stale task until it is reviewed — has to spell the refusal as `run = "false"`. The condition belongs to `when`; the command is a stand-in for a refusal the config cannot state directly.


== Scope

Make `run` optional on a gating action: `gate = true` with a `message` and no `run` refuses the flip unconditionally whenever the hook's `when` matches.

```toml
[[on.flip]]
when = 'into("wip") && stale(30)'
actions = [{ gate = true, message = "re-read the scope, `mt review` it, then start again" }]
```

Touches the kind resolution in `crates/mindtape/src/config_hooks.rs` with `config_hooks_tests.rs`, the dispatch in `crates/mindtape-cli/src/flip_hooks.rs` with `flip_hooks_tests.rs`, `schema/config.schema.json`, `www/content/hooks.typ` and `CHANGELOG.md`. Red test: a matching run-less gate refuses the flip and prints its message; a non-matching one lets it through.

Dogfood: drop `run = "false"` from this repo's review-before-start gate in `.mindtape/config.toml`.

