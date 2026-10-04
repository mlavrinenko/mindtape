#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Spread the FNV hash through a mixer before taking its leading digits",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[recurring and numbered tasks get 3-character hashes, not 5],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 4),
)

== Problem

+ Reviewing the stem-hash commit 45d958bd against examples/large
+ 🔴 #raw("crates/mindtape/src/handle.rs"): \`encode\` takes FNV-1a 64's most significant digits first, and FNV-1a spreads a stem's last bytes poorly into its top bits, so stems that differ only at the end share long prefixes.

Hash lengths, `mt ls '' --columns slug#` in `examples/large`, then a simulation with a splitmix64 finaliser:
```
task-00001..02000      fnv: 1978×5, 21×4, 1×8, 1×3   mixed: 1854×3, 142×4, 4×5
standup-2026-10-01..30 fnv: 29×5, 1×3                mixed: 30×3
```
Recurring stems (`{slug}-{date}`, `recur.rs:45`) and `-2` dedup suffixes have this shape.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/handle.rs").

== TO-BE

- 🟢 Sequential and dated stems get hashes as short as unrelated ones: 2000 `task-NNNNN` stems are mostly 3 characters.
- 🟢 Every hash is minted anew once; nothing pre-release keeps the old ones.


== Changes

- `crates/mindtape/src/handle_tests.rs`: red test first: lengths for 30 dated stems stay at 3; re-pin the literal code.
- `crates/mindtape/src/handle.rs`: run a 64-bit finaliser (splitmix64 or fmix64) over the FNV result before `encode`.


== Rollback

- Revert the commit; the unmixed codes and their pinned literal return.


== Out of scope

- Reserving stems and filter words, which its own finding does.
- Swapping FNV for another hash crate: a finaliser is enough.


== Dogfood

Recurring tasks in this repo and `examples/recurring` list 3-character hashes.
