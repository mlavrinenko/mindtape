#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print nothing from a passing just check but its verdict",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  estimate: (src-files: 4),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[a failure stands out when a pass prints nothing],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
)

== Problem

+ Gating nine docs branches in a row.
+ 🔴 #raw("Justfile"): A passing \`just check\` prints about 40 lines that a pass already implies, and the CRAP report lists functions as new on every run.

Two passing runs printed 40 and 37 lines: cargo-machete's "Good job!", linecop's "All files within size limits.", two nextest run banners, the llvm-cov cfg note and a CRAP table. After `just crap-baseline` the table still ends:
```
↑ 0 regressed · ↓ 0 improved · ★ 2 new · ↔ 0 moved · · 1587 unchanged · — 2 removed
```
The two `From` impls in `crates/mindtape-cli/src/task_ref.rs` share the name `TaskRef::from`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 A passing `just check` prints one verdict line: `check: 28 gates passed (7 run, 21 cached)`.
- 🟢 A failing gate prints its tool's full output and the path of its log, and the verdict names it.


== Changes

- `Justfile`: the `mmz` recipe captures each gate's output under `target/check/` through `.just/scripts/mmz-arm.sh`, and `check` prints the verdict through `.just/scripts/check.sh`.


== Rollback

Revert the commit; each gate prints its full output again.

== Out of scope

- What any gate checks. Only what it prints on a pass changes.
- Running the tests once instead of twice for coverage, its own decision.
- Keying the two `TaskRef::from` impls apart in the CRAP report. A pass no longer prints the report.


== Dogfood

Every `just check` in this repo, by people and agents, prints a few lines on a pass.

