#import "@local/mindtape:0.2.0": *
#import "../goals.typ": helps, neutral
#show: task.with(
  title: "Harden session tokens",
  kind: "milestone",
  priority: ice(impact: 7, confidence: 0.8, ease: 5),
  status: wip(2026, 9, 29),
  estimate: (src-files: 0),
  tags: ("auth", "security"),
  x-goals: (
    secure: helps[an expired session can't mint a new token],
    fast: neutral,
  ),
)

== Problem
+ Run `curl -X POST -b session=<expired-id> localhost:8080/token/refresh`.
+ 🔴 It returns `200` with a new access token, signed with a key that is 214 days old.

== TO-BE
- 🟢 An expired session can't refresh its token, and the signing key rotates every 90 days.

== Changes
- `tasks/*.typ`: the child tasks, linked to this one with `parent`. This milestone changes no file itself.

== Out of scope
- Throttling failed logins, which waits on the Redis client.
