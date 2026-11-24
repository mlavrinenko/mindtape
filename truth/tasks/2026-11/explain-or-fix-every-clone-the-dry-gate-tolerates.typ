#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Explain or fix every clone the dry gate tolerates",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("stateless", "tooling", "gating"),
  assignee: "agent",
  status: done(
    2026,
    11,
    24,
  )[1,287 clones explained, 7 fingerprints fixed, 28 extractions left as debt],
  links: related(
    "gate-short-repeats-on-jscpds-native-duplicates-and-baseline.typ",
  )[the gate reads the baseline this task explains],
)

== AS-IS

The dry gate tolerates 1,294 clones and explains none of them.

+ Run `jq '.fingerprints | length' .config/jscpd-baseline.json`.
+ 🔴 It prints 1294: every clone `just lint dry` lets through, and none carries a reason.
+ Re-mint the baseline with `just lint dry-baseline`.
+ 🔴 jscpd 5.4.3 adds 83 clones the fork missed and drops 161. A re-mint moves the set unexplained. 1,068 are test-to-test, 195 src-to-src, 31 mixed.
+ Read `.config/jscpd-baseline.json`.
+ 🔴 It maps a fingerprint to `1` and nothing else, so a reader cannot tell a coincidence from a clone nobody got to.
+ Read the close rule in `docs/src/contributing/signal-sweep.typ`.
+ 🔴 It says a clone closes by extraction or by a `jscpd:ignore` marker whose comment says why. `just lint dry-baseline` re-mints the baseline with no reason at all.

== TO-BE

- 🟢 Every fingerprint in `.config/jscpd-baseline.json` has a reason in `.config/jscpd-baseline-reasons.json`, or leaves the file because the clone got a fix.
- 🟢 `just lint dry` fails on a baseline fingerprint the reasons file does not explain, and a re-mint cannot add one silently.
- 🟢 Each fixed clone drops its fingerprint, and the reasons file drops its reason.

```
$ just lint dry
dry: 1287 clones tolerated, every one explained

$ just lint dry
dry: 1 baseline entry has no reason: 0007733b7e5896df
hint: fix the clone, or add its reason to .config/jscpd-baseline-reasons.json
```

== Changes

- `.config/jscpd-baseline.json`: 1,287 fingerprints after the six source fixes dropped seven.
- `.config/jscpd-baseline-reasons.json`: one reason per fingerprint, 1,182 distinct sentences. Twenty-eight name an extraction as a separate task. The rest say why no helper removes the clone.
- `.just/lint.just` and `.just/scripts/dry-reasons.sh`: `dry` and `dry-baseline` fail on a baseline fingerprint with no reason and on a reason with no fingerprint.
- `crates/mindtape-cli/src/cmd/set.rs` and `add/values_ops.rs`: `parse_assignment` calls `field_op`.
- `crates/mindtape-cli/src/cmd/init/preset.rs`: `read_dir_paths` holds the missing-dir return and the `read_dir` loop.
- `crates/mindtape-cli/src/cmd/rm/gitless/mod.rs`: `retire_plan` calls `dest_rel`.
- `crates/mindtape-cli/src/render/fields.rs`: `cell_from` holds the `CellSpec` match.
- `crates/mindtape/src/graph/ready.rs`: `target_stem` leaves the tree, and the caller uses `normalize_link`.
- `crates/mindtape/src/sort.rs`: `cmp_num_desc` delegates to `cmp_num_asc`.
- `crates/mindtape/src/loader/visible_git.rs`: the `rev-parse` fallback stays duplicated, and its reason records the extraction as a separate task.
- `.mmz/config.yaml`, `.config/outdatty.yaml`, `docs/src/contributing/signal-sweep.typ` and `CHANGELOG.md`: the gate's inputs, the script's group, the close rule and the entry.

== Out of scope

- The gate on short repeats, which is its own task.
- The short repeats the repeats gate refuses, which the test-helper dedupe tasks fix.
- The clones the gate already refuses, which never reach the baseline.

== Dogfood

This repo's `just lint dry` fails on a baseline entry with no reason, and the baseline shrank from 1,294 to 1,287 as the six source fixes landed.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a tolerated clone carries a reason or leaves the baseline, so the gate cannot hide a clone nobody reviewed].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
