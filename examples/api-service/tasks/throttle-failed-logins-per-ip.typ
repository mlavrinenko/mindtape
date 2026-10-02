// docs:slice showcase
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Throttle failed logins per IP",
  kind: "feat",
  status: proposed(2026, 9, 28),
  estimate: (src-files: 2),
  links: depends-on("add-redis-client.typ"),
)

== Problem
+ Run `for i in $(seq 50); do curl -so /dev/null -w '%{http_code} ' -d user=ada -d password=x localhost:8080/login; done`.
+ 🔴 It prints `401` fifty times. A script can guess passwords as fast as the server answers.

== TO-BE
- 🟢 After 10 failed logins from one IP in 10 minutes, `/login` returns `429` with a `Retry-After` header.

== Changes
- `src/auth/limiter.rs`, new: count failures per IP in Redis, each count expiring after 10 minutes.
- `src/auth/login.rs`: ask the limiter before checking the password, and record each failure.

== Alternatives
- A counter in process memory. Each replica would count on its own, so three replicas would allow 30 tries.

== Out of scope
- Locking accounts by username, which needs its own unlock flow.
// docs:end
