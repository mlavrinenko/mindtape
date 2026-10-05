#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give Violation a kind and match on it in place of message text",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
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
      sound-codebase: helps[a reworded message cannot silently change a check's behavior],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== Problem

+ Filing the follow-ups that read the check results of \`mt add\` templates.
+ 🔴 #raw("crates/mindtape/src/validate_links.rs"): \`is\_unresolved\` decides that a breach is an unresolved link target by testing the start and end of its message text

```
pub(crate) fn is_unresolved(violation: &Violation) -> bool {
    let message = &violation.message;
    message.starts_with("link target `") && message.ends_with("` not found")
}
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/violation.rs").

== TO-BE

- 🟢 `Violation` carries a small kind, `Kind::UnresolvedLink` for a link target not in the set and `Kind::Other` for the rest, which the link check sets where it builds the breach.
- 🟢 The template check drops a breach by its kind, so rewording the message cannot change which breaches it drops.


== Changes

- `crates/mindtape/src/validate_links_tests.rs`: the red test first, a reworded message still classified as unresolved by its kind.
- `crates/mindtape/src/violation.rs`: the kind enum and the `Violation` field.
- `crates/mindtape/src/validate_links.rs`: set the kind on the unresolved-target breach and match on it in `is_unresolved`.


== Rollback

- Revert the commit.


== Out of scope

- Every other breach kind. They gain a kind when a consumer needs to match one.


== Dogfood

The template check in this repo drops unresolved links by kind.
