#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Rotate the JWT signing key",
  kind: "dev",
  priority: ice(impact: 8, confidence: 0.9, ease: 8),
  status: proposed(2026, 9, 21),
  estimate: (src-files: 1),
  tags: ("auth", "security"),
  due: on(2026, 10, 5),
  recur: every(days: 90),
  links: related(
    "throttle-failed-logins-per-ip.typ",
  )[both change how login tokens are checked],
)

== Problem
+ Run `grep JWT_SIGNING_KEY .env.production`.
+ 🔴 The key is 214 days old, and a leaked key would sign valid tokens until someone noticed.

== TO-BE
- 🟢 The production key is at most 90 days old, and tokens signed with the previous key stay valid for 24 hours.

== Changes
- `src/auth/token.rs`: verify a token against the current and the previous key.
- `scripts/rotate-key`, new: generate a key and print the two environment lines to deploy.

== Rollback
- Deploy the previous `JWT_SIGNING_KEY` again. Tokens signed with the new key stop verifying, and their users log in once more.

== Out of scope
- Moving the key into a secrets manager.
