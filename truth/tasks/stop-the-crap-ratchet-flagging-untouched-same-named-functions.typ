#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop the CRAP ratchet flagging untouched same-named functions",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a CRAP failure names a function the branch changed],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a re-mint stops hiding real regressions behind line shifts],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Integrating two agent branches from the pre-release docs review
+ 🔴 #raw("crap-baseline.json"): two branches that never touched crates/mindtape-cli/src/diag/from\_lib.rs each failed the crap gate on a Diag::from regression there and re-minted the whole baseline

```
cache-imports: regression on Diag::from in crates/mindtape-cli/src/diag/from_lib.rs; re-minted, 119-line baseline diff
check-json:    regression on Diag::from in from_lib.rs; re-minted, 310-line baseline diff
```
Replayed, cache-imports regressed `TaskCache::get` and `TaskCache::open`; `Diag::from` was the table's first row, `NEW`. The scan's paths are absolute and the baseline's relative, so cargo-crap's exact `(file, function, line)` pass never fires, and its suffix pass ties on the 17 functions sharing a name within a file. They report `NEW` plus removed on every run, are never compared, and sort above real regressions.

== TO-BE

- 🟢 A branch that leaves a file untouched never fails the crap gate on a function in that file.
- 🟢 A real regression in one of several same-named functions still fails, naming it.

  ```sh
  # crap.sh compares on (file, function, ordinal within file), not (file, function, line)
  ```


== Changes

- `.just/scripts/_crap-ratchet-lib.sh`: before the ratchet, give the baseline the scan's root and, per `(file, function)` group in line order, the current line at the same position, so the exact pass keys on the ordinal.
- `.just/scripts/_crap-ratchet-control.sh`: a one-file tree with two same-named functions; a shift alone pairs both, a rise in the second fails naming it. `crap.sh` runs it first.
- Upstream: an issue for cargo-crap to pair a tied suffix group by position.


== Rollback

- Revert the commit.


== Out of scope

- Lowering any threshold.


== Dogfood

Every branch integrated here stops carrying an unrelated baseline re-mint.

