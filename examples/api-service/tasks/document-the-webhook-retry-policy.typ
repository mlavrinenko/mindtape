#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Document the webhook retry policy",
  kind: "docs",
  priority: ice(impact: 5, confidence: 0.7, ease: 8),
  status: proposed(2026, 9, 30),
  estimate: (src-files: 1),
  tags: ("billing", "docs"),
  due: on(2026, 10, 3),
)

== Problem
+ Open `docs/webhooks.md`.
+ 🔴 It does not say how often or how long the service retries a failed delivery, so customers ask support.

== TO-BE
- 🟢 The page states the retry schedule and when the service gives up.

== Source of truth
- `RETRY_DELAYS` in `src/webhooks/dispatch.rs`.

== Changes
- `docs/webhooks.md`: add a Retries section with the schedule and the final failure state.

== Out of scope
- Changing the schedule.
