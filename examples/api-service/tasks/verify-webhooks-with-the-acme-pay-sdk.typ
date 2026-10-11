#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Verify webhooks with the acme-pay SDK",
  kind: "dev",
  priority: ice(impact: 5, confidence: 0.9, ease: 8),
  status: proposed(2026, 9, 24),
  estimate: (src-files: 2),
  waits-on: ("locked:acme-pay@2.4.0",),
  tags: ("billing",),
)

== Problem
+ Open `src/billing/webhook.rs`.
+ 🔴 It checks each payment webhook's signature by hand, because `acme-pay` 2.3 rejects a valid signature whose timestamp header ends in a space. 2.4.0 fixes that, and our hand-rolled check is one more place to get HMAC wrong.

== TO-BE
- 🟢 `src/billing/webhook.rs` calls `acme_pay::Webhook::verify` and holds no signature code of its own.

== Changes
- `src/billing/webhook.rs`: replace the hand-rolled HMAC check with `Webhook::verify`.
- `Cargo.toml`: drop the `hmac` and `sha2` crates the check used.

== Rollback
- Revert the commit. The hand-rolled check comes back unchanged, and `acme-pay` stays at whatever the lockfile pins.

== Out of scope
- Upgrading `acme-pay`. The dependency bot opens that pull request, and `waits-on` holds this task until it merges.
