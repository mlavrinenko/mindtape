#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

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
  status: done(
    2026,
    9,
    30,
  )[bounded the whitespace scans in `whole\_links\_edit` and `sweep\_operator`. `just mutants -f crates/mindtape-cli/src/rm\_unlink.rs` reports 94 tested, 0 missed, 0 timeouts.],
)

== AS-IS

+ Filing while rebuilding `mutants.out` from the `--iterate` sweep for `rebuild-the-stale-mutants-outcomes-file`.
+ 🔴 #raw("crates/mindtape-cli/src/rm_unlink.rs"): five arithmetic mutants stop the whitespace-scan loops from advancing. Each then hangs until the 120 s test timeout instead of returning a verdict.

`mutants.out/timeout.txt`: `234:24 +=`->`*=` in `whole_links_edit`. `253:16` and `258:18` `-=`->`/=` plus `265:15` and `270:21` `+=`->`*=` in `sweep_operator`.

This fell outside the task that surfaced it, so I filed it here rather than
leave it in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/rm_unlink.rs").

== TO-BE

- 🟢 A `for` over a fixed range bounds every whitespace scan in `whole_links_edit` and `sweep_operator`, so a mutant that stops advancing exits instead of hanging.
- 🟢 `just mutants -f crates/mindtape-cli/src/rm_unlink.rs` reports 0 timeouts.

== Changes

- `crates/mindtape-cli/src/rm_unlink.rs`: wrap each `while <cond> { cursor += 1 }` / `-= 1` scan in a `for _ in 0..source.len().saturating_add(1)` whose body breaks when the condition fails. This is the pattern already used for `config_hooks.rs` and `loader.rs`, and it reaches the bound even when the counter stops moving.

== Rollback

Revert the `rm_unlink.rs` hunk. The scans go back to `while` loops and the five mutants go back to timing out, with nothing else changed.

== Out of scope

- The survivors this file leaves missed (`sweep_operator` and `whole_links_edit` arithmetic): the sweep pins or files them, not this task.

== Dogfood

Not applicable: the bound does not change any user-visible behavior. This repo's mutation sweep reports a verdict instead of a timeout.
