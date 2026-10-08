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
  status: proposed(2026, 10, 8),
)

== Problem

+ Integrating two agent branches from the pre-release docs review
+ 🔴 #raw("crap-baseline.json"): two branches that never touched crates/mindtape-cli/src/diag/from\_lib.rs each failed the crap gate on a Diag::from regression there and re-minted the whole baseline

```
cache-imports: regression on Diag::from in crates/mindtape-cli/src/diag/from_lib.rs; re-minted, 119-line baseline diff
check-json:    regression on Diag::from in from_lib.rs; re-minted, 310-line baseline diff
```
`from_lib.rs` has 11 `Diag::from` entries in the baseline, told apart only by `line` (19, 44, 50, 89, 158, 164, 249, 255, 276, 321, 329). Unconfirmed: how cargo-crap 0.3.1 pairs same-named entries when lines shift.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/crap.sh").

== TO-BE

- 🟢 A branch that leaves a file untouched never fails the crap gate on a function in that file.
- 🟢 A real regression in one of several same-named functions still fails, naming it.

  ```sh
  # crap.sh compares on (file, function, ordinal within file), not (file, function, line)
  ```


== Changes

- First, reproduce: on a scratch branch insert a blank line above `from_lib.rs`'s first `impl From`, run `just crap`, and record what it pairs. Read cargo-crap 0.3.1's baseline matching.
- `.just/scripts/crap.sh`: if pairing is the cause, compare with a key that survives line shifts, or report upstream to cargo-crap with the repro and pin the fix.


== Rollback

- Revert the commit.


== Out of scope

- Lowering any threshold.


== Dogfood

Every branch integrated here stops carrying an unrelated baseline re-mint.

