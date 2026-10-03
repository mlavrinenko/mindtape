#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep each record's stem on import so links between records resolve",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[an export of a real project imports back whole],
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

+ Round-tripping the example projects through `mt ls '' --json` and `mt import` for /import-export/.
+ 🔴 #raw("crates/mindtape-cli/src/import.rs"): `mt import` names each file from the record's title and ignores its `id`, so a link to a record whose stem differs from its title no longer resolves, and the record holding the link is skipped.

```
$ mt -w examples/api-service ls '' --json > export.json
$ mt init --no-input && mt import export.json
2 filed, 1 skipped
  add-redis-client: filed
  reject-expired-sessions-on-token-refresh: filed
  throttle-failed-logins-per-ip: skipped
    - depends-on: link target `tasks/add-redis-client.typ` not found
$ ls tasks
add-a-redis-client-to-the-app-state.typ  reject-expired-sessions-on-token-refresh.typ
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/import.rs").

== TO-BE

- 🟢 A record with an `id` is written to that stem, disambiguated as a title stem is, so its links and the links to it resolve after an export and import.
- 🟢 A record with no `id`, as the jq adapters emit, is still named from its title.


== Changes

- `crates/mindtape-cli/src/import.rs`: take the stem from `id` when it is set, else from the title.
- `crates/mindtape-cli/src/import_tests.rs`: import a record whose `id` differs from its title, with a link to it.


== Rollback

Revert the commit; import names every file from its title again.

== Out of scope

- Rewriting link targets when a stem is disambiguated: a clash with an existing file is rare and the report names it.


== Dogfood

Not applicable; this repo imports nothing.
