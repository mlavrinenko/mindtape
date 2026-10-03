#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Send a welcome email after signup",
  kind: "feat",
  priority: ice(impact: 6, confidence: 0.6, ease: 5),
  status: proposed(2026, 9, 26),
  estimate: (src-files: 4),
  tags: ("notifications",),
  due: on(2026, 10, 30),
  start: on(2026, 10, 15),
)

== Problem
+ Sign up with a new address.
+ 🔴 No email arrives, so a user who mistyped the address never learns it.

== TO-BE
- 🟢 A signup sends one welcome email with a confirmation link within a minute.

== Changes
- `Cargo.toml`: add the `lettre` crate.
- `src/config.rs`: read `SMTP_URL` from the environment.
- `src/notify/welcome.rs`, new: render and send the email.
- `src/auth/signup.rs`: queue the email after the user row commits.

== Alternatives
- Send the email inside the signup request. A slow SMTP server would make signup slow.

== Out of scope
- Marketing emails.
