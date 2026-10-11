#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hold a new function to the crap ratchet from the commit that adds it",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a function's CRAP score is ratcheted from the commit that adds it, not from the next re-mint],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[an undertested new function can't slide in under the threshold with no baseline to hold it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ On main at d811e6e7, run `just crap-baseline` and diff `crap-baseline.json` against the committed one.
+ 🔴 Functions that earlier commits added have no entry, among them `date_tokens` (crates/mindtape-cli/src/cli/tree_json.rs), `handle` (crates/mindtape-cli/src/cmd/add/output.rs), `stem_hashes_with` (crates/mindtape-cli/src/util.rs) and `DateToken::render` (crates/mindtape/src/date/mod.rs). The ratchet in `.just/scripts/_crap-ratchet-lib.sh` compares only functions the baseline lists, so until someone re-mints, nothing stops these from getting worse.
+ The branch for `refuse-a-commit-range-that-leaves-a-task-it-names-in-progress-at-its-tip` re-minted the baseline to pass `just crap`, rewriting about 280 lines, so its one real rise (`check_commit`, 7 to 8) sat among unrelated changes.


== TO-BE

- 🟢 `just crap` fails on a scored function with no baseline entry, naming it, with the hint to run `just crap-baseline` in the same commit.
- 🟢 A re-mint's diff holds only the functions that commit added or changed.


== Changes

- `.just/scripts/_crap-ratchet-lib.sh`: a scored function absent from the baseline fails the ratchet, as a rise does. The red test first, in the ratchet's control script (`.just/scripts/_crap-ratchet-control.sh`).
- `docs/src/contributing/crap-baseline.typ` (or wherever the baseline is documented) and its generated md: a new function means a re-mint in the same commit.
- `crap-baseline.json`: re-minted once so main starts complete.


== Rollback

- Revert the commit. The baseline format does not change.


== Out of scope

- `.just/scripts/crap-baseline-check.sh`: it reads the committed file without scores, so it cannot see a missing function.
- The CRAP threshold itself.


== Dogfood

The next commit here that adds a function fails `just check` until it re-mints the baseline beside the code.

