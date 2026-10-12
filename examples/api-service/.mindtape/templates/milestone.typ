#import "@local/mindtape:0.2.0": *
#import "../goals.typ": helps, hurts, neutral
#show: task.with(
  title: slot(
    "title",
    prompt: "Starts with a verb: Harden session tokens",
    example: "Harden session tokens",
  ),
  kind: "milestone",
  estimate: (src-files: 0),
  x-goals: (
    secure: slot(
      "secure",
      kind: "raw",
      default: "neutral",
      prompt: "helps[how], hurts[how] or neutral",
      example: "helps[an expired session can't mint a new token]",
    ),
    fast: slot(
      "fast",
      kind: "raw",
      default: "neutral",
      prompt: "helps[how], hurts[how] or neutral",
    ),
  ),
)

== Problem
#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps that reproduce it. Open the failing step with 🔴.",
  example: ```typ
  + Run `curl -X POST -b session=<expired-id> localhost:8080/token/refresh`.
  + 🔴 It returns `200` with a new access token.
  ```,
)

== TO-BE
#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it works once the milestone closes",
  example: ```typ
  - 🟢 An expired session can't refresh its token.
  ```,
)

== Changes
- `tasks/*.typ`: the child tasks, linked to this one with `parent`. This milestone changes no file itself.

== Out of scope
#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "What this milestone leaves alone",
  example: ```typ
  - Throttling failed logins.
  ```,
)
