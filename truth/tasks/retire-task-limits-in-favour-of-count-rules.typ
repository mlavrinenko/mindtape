#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Retire task limits in favour of count rules",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 10),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a WIP cap is a rule like any other, overridable by name],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[about 430 lines of limits config and tests go],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
  links: depends-on("add-sum-and-count-aggregates-to-filters.typ"),
)

== Problem

+ `.mindtape/config.toml` declares two `[[tasks.limits]]` WIP caps.
+ 🔴 `[[tasks.limits]]` is its own table and checker (`crates/mindtape/src/config_limits.rs`) for what `count(status == "wip") <= 1` says as a rule, and it cannot be merged by name across packs like a rule.


== TO-BE

- 🟢 `[[tasks.limits]]` is gone, with no alias; a WIP cap is a `[[tasks.rules]]` entry over `count(...)`, refusing a flip at `error` exactly as the limit did.
- 🟢 A breach names the tasks in the counted set, as the limit report did.


== Changes

- `crates/mindtape-cli/tests/flip_hooks_e2e.rs` or a new `wip_cap_e2e.rs` (red first): a flip over a `count` rule refused, listing the set.
- `crates/mindtape/src/config_limits.rs`, `config_limits_tests.rs`: delete.
- `crates/mindtape/src/config.rs`, `lib.rs`, `crates/mindtape-cli/src/check.rs`, `flip.rs`: drop the limits pass.
- `.mindtape/config.toml`, `examples/learn-piano/.mindtape/config.toml`: the rules.
- `www/content/config.typ`, `docs/src/skill.typ`: rules instead of limits.
- `CHANGELOG.md`: the removal.


== Alternatives

- Keep limits as sugar over rules: two spellings of one check.


== Out of scope

- Other set-level checks: they arrive as rules when needed.


== Dogfood

This repo's two caps move to rules in the same commit.

