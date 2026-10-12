#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: slot(
    "title",
    prompt: "Starts with a verb: Throttle failed logins per IP",
    example: "Throttle failed logins per IP",
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    default: 3,
    prompt: "How much judgment it takes, 0 to 9. A rename is 1, a design call 3 and up",
  ),
  estimate: (
    src-files: slot(
      "src-files",
      kind: "int",
      prompt: "How many source files the change touches",
      example: 2,
    ),
  ),
)

== Problem
#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps that reproduce it. Open the failing step with 🔴.",
  example: ```typ
  + Send 50 wrong passwords for one account from one IP.
  + 🔴 Every attempt reaches the password check, and none is slowed down.
  ```,
)

== TO-BE
#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it works once this lands",
  example: ```typ
  - 🟢 After 5 failed logins in a minute, an IP gets `429 Too Many Requests`.
  ```,
)

== Changes
#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: its `path`, then what changes there",
  example: ```typ
  - `src/auth/login.rs`: count failures per IP and refuse past the limit.
  - `src/state.rs`: keep the failure counters in `AppState`.
  ```,
)

== Out of scope
#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "What this task leaves alone",
  example: ```typ
  - Locking the account itself. This task slows down one IP.
  ```,
)
