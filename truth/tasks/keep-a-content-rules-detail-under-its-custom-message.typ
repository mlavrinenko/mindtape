#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep a content rule's detail under its custom message",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "gating", "cli"),
  estimate: (src-files: 11),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a breach says both what to do and which sentence to do it to],
      enforced-practice: helps[a rule's advice no longer hides where the rule fired],
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
  status: done(2026, 10, 25),
)

== Problem

+ Give a `density` content rule a `message`, and break it in one sentence of a task.
+ Run `mt check`.
+ 🔴 It prints only the message, ``` tasks/fix-login-bug.typ:18: body.content.rules.no-open-questions: ask the user and write the answer in ```, and never says which sentence broke it. Without `message` it names the sentence and drops the advice.


== TO-BE

- 🟢 When a content rule sets `message`, the breach keeps the check's own words on a `note:` line under it, for every assertion kind.
- 🟢 `mt check --json` lists that line in a `notes` array beside `hints`.
- 🟢 A write that a content rule refuses, such as `mt set` or `mt flip`, prints the same note.

```text
error: tasks/fix-login-bug.typ:18: body.content.rules.no-open-questions: ask the user and write the answer in, since a task leaves no question open
note: section `TO-BE` sentence 2 ("Should it also cover the board?") must carry at most 0 matches of `\?(\s|$)` (found 1)
```


== Changes

- `crates/mindtape-cli/tests/it/check_content_detail_e2e.rs`: the red test first, text and JSON output of a breach with and without `message`.
- `crates/mindtape/src/validate/violation.rs`: a `detail: Option<String>` field, the check's own words when a configured message replaced them.
- `crates/mindtape/src/body/content_gate/mod.rs`: set `detail` to the default message when `message` replaces it.
- `crates/mindtape-cli/src/diag/from_lib.rs`: render `detail` as a note.
- `crates/mindtape-cli/src/diag/json.rs`: a `notes` array in each problem.
- `crates/mindtape/src/{graph/children,template/check,validate/mod,validate/links,config/comments,rules/eval}.rs`: `detail: None` where each builds a breach.


== Alternatives

- An indented line with no prefix. Every line `mt` prints to stderr starts with its kind, so a script that reads them by prefix keeps working.
- Append the detail to the message after a colon. The line gets long, and a JSON reader can't split the two.


== Out of scope

- Header rules, whose engine words a breach by the rule's shape and holds no detail of its own.
- `mt import`'s per-record report, which prints one line per problem.


== Dogfood

The `no-open-questions` rule this repo plans names the open question under its advice.

