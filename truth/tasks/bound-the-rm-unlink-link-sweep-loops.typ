#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop the rm_unlink whitespace scans hanging under mutation",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a mutation sweep reports a verdict instead of timing out on a loop that never advances],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 30),
)

== Problem

+ Filing while rebuilding \`mutants.out\` from the \`--iterate\` sweep for \`rebuild-the-stale-mutants-outcomes-file\`.
+ 🔴 #raw("crates/mindtape-cli/src/rm_unlink.rs"): five arithmetic mutants stop the whitespace-scan loops from advancing, so each hangs until the 120 s test timeout instead of returning a verdict.

`mutants.out/timeout.txt`: `234:24 +=`->`*=` in `whole_links_edit`; `253:16` and `258:18` `-=`->`/=` plus `265:15` and `270:21` `+=`->`*=` in `sweep_operator`.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/rm_unlink.rs").

== TO-BE

- 🟢 every whitespace scan in `whole_links_edit` and `sweep_operator` is bounded by a `for` over a fixed range, so a mutant that stops advancing terminates instead of hanging.
- 🟢 `just mutants -f crates/mindtape-cli/src/rm_unlink.rs` reports 0 timeouts.

== Changes

- `crates/mindtape-cli/src/rm_unlink.rs`: wrap each `while <cond> { cursor += 1 }` / `-= 1` scan in a `for _ in 0..source.len().saturating_add(1)` whose body breaks when the condition fails. This is the pattern already used for `config_hooks.rs` and `loader.rs`, and it reaches the bound even when the counter no longer moves.

== Rollback

Revert the `rm_unlink.rs` hunk; the scans go back to `while` loops and the five mutants go back to TIMEOUT, nothing else changes.

== Out of scope

- The survivors this file leaves missed (`sweep_operator` and `whole_links_edit` arithmetic): they are pinned or filed by the sweep, not by this task.

== Dogfood

Not applicable: the bound changes no user-visible behaviour; its only effect is that this repo's mutation sweep reports a verdict instead of a timeout.
