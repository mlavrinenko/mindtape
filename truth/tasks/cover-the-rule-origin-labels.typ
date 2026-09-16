#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cover the rule-origin labels",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "tests"),
  estimate: (files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the origin column is how a repo checks which pack supplied a rule, so its labels have to be pinned],
      cheap-adoption: helps[a repo adopting a pack trusts the merge only as far as it can see where each rule came from],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the one function over the CRAP threshold goes back under it],
      docs-match-code: neutral,
    ),
  ),
  kind: "test",
  assignee: "agent",
  status: proposed(2026, 9, 16),
)

== Problem

+ Run `just cover`, then `just crap`.
+ 🔴 One function is over the CRAP threshold of 30: `describe_rule_origin` (`crates/mindtape-cli/src/config_cmd.rs:156`) scores 56 on seven branches at 0% coverage.
+ Run `mt config show --origin` in a project that extends a pack.
+ 🔴 Every rule's origin label comes from that function, so a wrong label misreports which pack or layer a rule came from and no test would catch it.


== TO-BE

- 🟢 A test pins each of the six origin labels `mt config show --origin` can print: `embedded`, `xdg`, `dir`, `project`, `pack <name>` and `pack <path>`.
- 🟢 `just crap` reports no function over the threshold.


== Changes

- `crates/mindtape-cli/src/config_cmd_tests.rs`: red tests first, over a project whose rules arrive from an embedded pack, a path pack, an ancestor layer and the project itself, asserting the `--origin` line each rule gets. Where a case cannot be reached through `render_origin`, call `describe_rule_origin` directly and say why in the commit body.


== Out of scope

- Rewriting `describe_rule_origin`: a flat match over six variants reads fine; the coverage is what is missing.
- `just crap-baseline`: re-mint it only if the threshold arm still fails once the tests land.


== Dogfood

Built in: this repo reads `mt config show --origin` to see which rules come from the `mindtape-tasks` pack.

