#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give each mutation its own sweep stem",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
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
      sound-codebase: helps[one task per survivor instead of one per class of operator symbol],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 30),
)

== Problem

+ Re-filing the survivors the refreshed sweep named, while finishing \`rebuild-the-stale-mutants-outcomes-file\`.
+ 🔴 #raw("contrib/ask-refactoring.sh"): \`stem\_for\` maps every run of non-alphanumeric characters to a single hyphen, so distinct mutations of the same expression collapse onto one stem and the later finding is skipped as already filed.

`just ask-refactoring --write` reported `137 finding(s) swept`, filed 106 and skipped 31 (`stem is taken`). All twelve `rm_unlink.rs` `sweep_operator` survivors share the one stem `mutants-crates-mindtape-cli-src-rm-unlink-rs-replace-with-in-sweep-operator`; thirteen stems folded thirty-one survivors in total.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("contrib/ask-refactoring.sh").

== TO-BE

- 🟢 `stem_for` gives a distinct stem to each distinct mutation, so `<`, `>`, `>=`, `==` no longer sanitize to the same string.
- 🟢 A sweep over the same outcomes files one task per survivor, not one per operator class.

== Changes

- `contrib/ask-refactoring.sh`: encode the operator characters before the general substitution, or append a short hash of the mutation text to the stem, so `replace > with >=` and `replace > with <` differ. Check the chosen form against the existing `truth/tasks/*` stems, since dedup reads them.

== Rollback

Revert the encoding; stems return to the collapsed form and the extra findings stop being filed.

== Out of scope

- The 106 tasks already filed from this sweep: each names a real mutation on its line, so they stay as they are.

== Dogfood

The next sweep files one task per survivor, so the pin work is countable and a closed class does not hide its siblings.
