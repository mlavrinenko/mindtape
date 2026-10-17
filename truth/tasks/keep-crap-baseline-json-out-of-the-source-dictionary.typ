#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep crap-baseline.json out of the source dictionary",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[src-budget counts only the source a task plans],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17)[baselines left out of \@source],
)

== Problem

+ Running `mt check` after `just crap-baseline` on the at.toml registry branch
+ 🔴 #raw(".mindtape/config.toml"): `[dicts.source]` doesn't exclude `crap-baseline.json`, so every task that adds a function, and so must re-mint the baseline, counts one extra source file against `estimate.src-files`

The branch changes four source files, matching its estimate of 4, plus `crap-baseline.json`, and `mt check` warns:
```
rules.src-budget: move work not yet written into a new task; what is already written ships with this one, so closing it needs no split, and estimate.src-files stays as filed
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".mindtape/config.toml").

== TO-BE

- 🟢 A re-minted `crap-baseline.json` doesn't count toward `src-budget` or `tweak-stays-one-file`:
```toml
[dicts.source]
values = [
  # ...
  "!crap-baseline.json",
  "!.jscpd-baseline.json",
]
```


== Changes

- `.mindtape/config.toml`: add `"!crap-baseline.json"` (and `"!.jscpd-baseline.json"`) to `[dicts.source]`.


== Rollback

- Remove the two entries.


== Out of scope

- The `src-budget` rule itself, which counts right once the dictionary does.


== Dogfood

The next task that adds a function closes without a false `src-budget` warning.
