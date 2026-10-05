#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Require a refusal capture to name why it refused",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 10),
  tags: ("found", "www"),
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
      sound-codebase: neutral,
      docs-match-code: helps[a page that shows a refusal shows the refusal its prose explains],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== Problem

+ Reviewing the api-service capture commits a71dbf48 and 3afcd40a
+ 🔴 #raw("www/capture.sh"): a capture with \`--exit 1\` checks only the exit code, and every \`mt\` error exits 1, so a page can show an unrelated error under prose that names a rule

```
$ mt wip no-such-task; echo $?
error: no-such-task: no task with id "no-such-task"
Error: 1 task(s) had problems
1
$ mt wip add-redis-client; echo $?
error: tasks/reject-expired-sessions-on-token-refresh.typ:2: rules.one-wip: finish the task in progress before you start another (reported on reject-expired-sessions-on-token-refresh)
error: add-redis-client: 1 task(s) rejected; nothing was written
Error: 1 task(s) had problems
1
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/capture.sh").

== TO-BE

- 🟢 `capture --exit 1 --expect 'rules\.one-wip' one-task-at-a-time "$P" 'mt wip add-redis-client'` fails the build unless the output matches the pattern.
- 🟢 A capture with a non-zero `--exit` and no `--expect` fails the build, so every refusal a page shows names its reason.


== Changes

- `www/capture.sh`: add `--expect <regex>`, match it against the output with ANSI stripped, and require it with a non-zero `--exit`.
- `www/generate-api-service.sh`, `www/generate-config-runs.sh`, `www/generate-commit-rules.sh`, `www/generate-reference.sh`, `www/generate-self-validating.sh`, `www/generate-rule-packs.sh`, `www/generate-quickstart.sh`, `www/generate-model.sh`, `www/generate-guides.sh`: give each `--exit 1` capture the rule name or message its page explains.


== Rollback

- Revert the commit; refusal captures check the exit code alone again.


== Out of scope

- A separate `mt` exit code for a rule or gate refusal, distinct from a lookup or usage error. It would help scripts too, and is a product decision of its own.


== Dogfood

The /one-task-at-a-time/, /gate-done-on-checks/ and other problem pages stop building when their refusal changes reason.

