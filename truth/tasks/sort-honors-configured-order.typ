#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Default sort by configured order, not alphabetical",
  status: done(2026, 6, 19),
  priority: ice(impact: 8, confidence: 0.8, ease: 9),
  estimate: (hours: 2),
  relates-to: (link("numeric-priority-scale.typ"), link("list-handle-ordinal-check.typ")),
  tags: ("cli",),
)

== Summary

`--sort status` and `--sort priority` compare strings, ignoring the configured
lifecycle order (`status.values` index) and treating priority alphabetically; the
default load order is alpha-by-id. Make ordering meaningful so list ordinals (and
their handles) are stable and scannable.

== Scope

- Sort status by its `status.values` index, not lexicographically.
- Sort priority numerically (see numeric-priority-scale).
- Define a sensible default list order: status lifecycle, then priority
  descending, then due ascending.
