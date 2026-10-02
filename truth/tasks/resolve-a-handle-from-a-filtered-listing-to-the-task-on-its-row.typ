#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve a handle from a filtered listing to the task on its row",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent can act on the handle any listing prints],
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
  status: proposed(2026, 10, 3),
)

== Problem

+ Checking the mindtape skill's handle advice against the code.
+ 🔴 #raw("crates/mindtape-cli/src/util.rs"): A filtered mt ls numbers its own rows, but resolve\_task resolves every handle against the default view, so a filtered handle can name another task.

`mt ls 'has_tag("found")' --columns '#,title'` prints `7i   Add common engineering verbs to the imperative-title dictionary`. `mt eval 7i --json | jq -r .title` prints `Make the docs plain, problem-led and unable to drift`. With one check character, about one filtered handle in 26 passes the check against the wrong task. The rest are refused with `handle check mismatch`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/util.rs").

== TO-BE

- 🟢 A handle that any `mt ls` listing prints resolves to the task on that row, or the listing prints no handle.
- 🟢 A test pins a filtered handle whose check character matches the default view's task at that ordinal.


== Changes

- `crates/mindtape-cli/src/util.rs`: resolve a handle only to the task the printing listing showed, or refuse it.
- `crates/mindtape-cli/src/ls.rs`: print a handle only where it resolves to its row.


== Rollback

Revert the commit.

== Out of scope

- Raising the default `[handle] check-len`: it lowers the odds and leaves the cause.


== Dogfood

Agents here list with filters such as `mt ls ready`, and the mindtape skill tells them to pass stems until this lands.
