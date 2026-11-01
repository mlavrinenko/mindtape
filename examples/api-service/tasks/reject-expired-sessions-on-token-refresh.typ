#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Reject expired sessions on token refresh",
  kind: "fix",
  priority: ice(impact: 8, confidence: 0.9, ease: 9),
  status: wip(2026, 9, 30),
  estimate: (src-files: 1),
  tags: ("stateless",),
  links: parent("harden-session-tokens.typ"),
)

== AS-IS
+ Log in, then wait 24 hours for the session to expire.
+ Run `curl -X POST -b session=<expired-id> localhost:8080/token/refresh`.
+ 🔴 It returns `200` with a new access token.

```sh
$ curl -i -X POST -b session=<expired-id> localhost:8080/token/refresh
HTTP/1.1 200 OK
```

== Environment
- Staging, release `v1.8.2`.

== TO-BE
- 🟢 A refresh with an expired session returns `401`, so the client sends the user to the login page.

```sh
$ curl -i -X POST -b session=<expired-id> localhost:8080/token/refresh
HTTP/1.1 401 Unauthorized
```

== Changes
- `src/auth/session.rs`: check `expires_at` in `refresh` before issuing a token, with a test for the expired case.

== Out of scope
- Changing the 24-hour session lifetime.
