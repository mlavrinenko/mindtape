#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Numeric priority on a configured scale",
  status: proposed(2026, 6, 19),
  priority: "high",
  estimate: (days: 1),
  relates-to: (link("priority-bands.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Priority is `Option<String>` validated against a string enum, and the filter
returns it as a string — so comparisons and `--sort priority` are lexicographic:
high < low < normal alphabetically, a genuine mis-sort. Make priority numeric.
`[tasks.priority]` configures the scale: `kind = int | real`, `min`, `max`
(default 0..9). Break the string enum now, while 0.2 is pre-release.

== Scope

- DSL: `priority: 4`. Drop `fields.priority.values`; add the `[tasks.priority]`
  scale block.
- Model: priority becomes a number, validated within `[min, max]`.
- Filter and sort treat priority numerically (fixes the lexicographic bug);
  default ordering puts higher priority first.
- Propagate the change to default-config.toml, the dogfood `.mindtape/config.toml`,
  README, and the prelude.
- Migrate existing `truth/tasks` priorities to the numeric scale when landing.
