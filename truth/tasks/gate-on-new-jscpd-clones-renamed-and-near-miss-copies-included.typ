#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Gate on new jscpd clones, renamed and near-miss copies included",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  estimate: (src-files: 1),
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
      sound-codebase: helps[a copy with renamed variables or a shorter copied block fails the gate, not only a long verbatim one],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Run `just check-dry`. jscpd 5.0.15 at `-k 70` passes with no clone.
+ 🔴 It matches exact tokens, so a renamed copy slips past. jscpd 5.4.1 with `--ignore-identifiers --ignore-literals` finds 599 renamed clones at `-k 70`, such as the hand-written string-to-enum `Deserialize` impls in `crates/mindtape/src/body/content.rs` and `crates/mindtape/src/model/naming.rs`.
+ 🔴 A lower `-k` would catch shorter copies, but it fails the gate at once on every clone the code already has: 202 exact ones at `-k 50`. So the floor stayed at 70 tokens.


== TO-BE

- 🟢 `just check-dry` fails on a new clone of 50 tokens or more, exact, renamed or a near-miss function (`--similarity=0.8`), in tests as in production code.
- 🟢 The clones the code has today sit in `.jscpd-baseline.json` and pass. Editing inside one makes it new, so the edit extracts it or re-mints the baseline in plain sight.
- 🟢 A failure prints only the new clones, one `file:line` pair each. A clean run prints nothing.
- 🟢 `just dry-baseline` re-mints the baseline.

```sh
jscpd -f rust -k 50 --ignore-identifiers --ignore-literals --similarity=0.8 \
  --baseline .jscpd-baseline.json --fail-on-new-clones -r json -o "$out" crates
```


== Changes

- `flake.lock`: bump qahq, whose jscpd input moves to 5.4.1.
- `Justfile`: `check-dry` runs jscpd 5.4.1 against the baseline and prints only the new clones. A `dry-baseline` recipe re-mints the baseline.
- `.jscpd-baseline.json`: the clones the code has today.
- `.mmz/config.yaml`: the baseline is an input of `just check-dry`.


== Rollback

- Revert the commit. The gate goes back to exact clones of 70 tokens, and `flake.lock` to the old qahq.


== Out of scope

- Working down the baseline, which is its own task.
- Ranking clones by copies times size, which waits on upstream jscpd issue 1000 (clone classes).


== Dogfood

This repo's `just check` runs the gate on every commit.
