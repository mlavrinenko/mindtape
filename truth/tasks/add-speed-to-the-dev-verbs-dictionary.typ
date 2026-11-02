#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add speed to the dev verbs dictionary",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "mt"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a correct title is no longer refused],
      ),
      "readable-tasks",
      "implementable-tasks",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== AS-IS

+ Filing a task titled Speed up login in the api-service example
+ 🔴 #raw("crates/mindtape/src/dicts/dev-verbs.txt"): The imperative-title rule refuses a title that starts with Speed, a common verb in engineering task titles.

`mt add "Speed up login" ...` printed `error: speed-up-login: rules.imperative-title: start the title with an imperative verb`, and `rg -x speed crates/mindtape/src/dicts/` finds nothing.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/dicts/dev-verbs.txt").

== TO-BE

- 🟢 A title starting with Speed passes the imperative-title rule.

== Changes

- `crates/mindtape/src/dicts/dev-verbs.txt`: add `speed`.

== Rollback

Remove the line.

== Out of scope

- Auditing the dictionaries for other missing verbs.

== Dogfood

Not applicable: this repo has no Speed title today.
