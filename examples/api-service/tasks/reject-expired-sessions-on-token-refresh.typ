#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Reject expired sessions on token refresh",
  kind: "fix",
  status: wip(2026, 9, 30),
  estimate: (src-files: 1),
)

== Problem
+ Log in, then wait 24 hours for the session to expire.
+ Run `curl -X POST -b session=<expired-id> localhost:8080/token/refresh`.
+ 🔴 It returns `200` with a new access token.

== Environment
- Staging, release `v1.8.2`.

== TO-BE
- 🟢 A refresh with an expired session returns `401`, so the client sends the user to the login page.

== Changes
- `src/auth/session.rs`: check `expires_at` in `refresh` before issuing a token, with a test for the expired case.

== Out of scope
- Changing the 24-hour session lifetime.
