#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Parse the allowlist in any rustfmt layout in the shrink test",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "tests"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the allowlist test holds in every layout],
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
  status: done(2026, 10, 8),
)

== Problem

+ Running just check on a branch rebased over a one-entry allowlist
+ 🔴 #raw("crates/mindtape-cli/src/diag/allowlist_tests.rs"): entries() reads one quoted entry per line, so a list rustfmt folds onto one line reads as empty and the shrink test reports its entry as added

const UNCONVERTED: &[&str] = &["cmd/check"];
diag/allowlist.rs only shrinks; build these errors as a Diag instead: ["cmd/check"]


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/diag/allowlist_tests.rs").

== TO-BE

- 🟢 entries() returns the quoted entries of a single-line, multi-line or empty list.

== Changes

- `crates/mindtape-cli/src/diag/allowlist_tests.rs`: take the quoted strings between `= &[` and `];`.

== Rollback

- Revert the commit.

== Out of scope

- The list itself.

== Dogfood

The shrink test passes on a branch rebased over any list layout.
