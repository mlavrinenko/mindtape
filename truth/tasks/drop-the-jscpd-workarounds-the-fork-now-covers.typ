#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the jscpd workarounds the fork now covers",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "dry"),
  estimate: (src-files: 6),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the clone gate, the sweep and the baseline merge run on jscpd's own features, not on jq written around its gaps],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 28),
)

== Problem

+ Read `.just/scripts/lint-dry.sh`.
+ 🔴 It filters jscpd's JSON report to the `isNew` clones with `jq`, because under `--baseline` every reporter prints every clone.
+ Read `contrib/ask-refactoring/dry.sh` and `contrib/ask-refactoring/repeats.sh`.
+ 🔴 Both join jscpd's clone pairs into families by hand, one by shared location and one by fragment text, waiting on clone classes (`jscpd-clone-classes` in `.config/canibump.toml`).
+ Read `.just/scripts/jscpd-baseline-merge.sh`.
+ 🔴 It is a merge driver of our own for `.config/jscpd-baseline.json`, because git's text merge conflicts when two branches each accept a clone.


== TO-BE

- 🟢 jscpd v5.4.1-nix.2, the owner's fork that qahq pins, does all three: `--new-only` prints only new clones, the json report lists `classes`, and `jscpd --merge-baseline %O %A %B` is the merge driver.
- 🟢 `just lint dry` calls jscpd with no script around it, both sweep arms read `classes`, and `just install-hooks` registers jscpd as the driver.


== Changes

- `flake.lock`: qahq at the commit that pins jscpd v5.4.1-nix.2.
- `.just/lint.just`: the `dry` gate runs `jscpd --new-only --fail-on-new-clones 0`, with the header of `lint-dry.sh`.
- `.just/scripts/lint-dry.sh`, `.just/scripts/jscpd-baseline-merge.sh`: deleted.
- `contrib/ask-refactoring/dry.sh`, `contrib/ask-refactoring/repeats.sh`: read jscpd's `classes`.
- `Justfile`, `.gitattributes`: register and assign `jscpd --merge-baseline`.
- `crates/mindtape-cli/tests/it/jscpd_baseline_merge_e2e.rs`: merge through the jscpd driver.
- `.config/canibump.toml`: drop the `jscpd-clone-classes` blocker, which qahq now tracks.
- `.mmz/config.yaml`, `.config/outdatty.yaml`, `docs/src/`: drop the deleted scripts.


== Rollback

- Revert the commits. qahq's older jscpd still runs the old scripts.


== Out of scope

- Gating short repeats in `just lint dry` and dropping the repeats arm, a policy call the classes make possible.
- `.just/scripts/outdatty-lock-merge.sh` and `.just/scripts/crap-baseline-merge.sh`, which work around other tools.


== Dogfood

This repo's `just lint dry`, `just ask-refactoring` and every rebase of `.config/jscpd-baseline.json` run on the fork's features.
