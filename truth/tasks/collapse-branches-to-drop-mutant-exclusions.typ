#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "collapse redundant branches to drop mutant exclusions",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("tests", "tooling"),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down that accumulated the exclusion list
  ],
  status: done(
    2026,
    7,
    21,
  )[six refactors, six exclusions dropped (10 → 4); in-diff sweep 19 mutants → 18 caught / 1 unviable / 0 missed],
)

== Summary

Seven of the ten `exclude_re` entries in `.cargo/mutants.toml` describe an
*equivalent mutant caused by redundant code*, not an equivalent mutant caused by
the problem's nature. A comparison that a callee already short-circuits, a
`.by_ref()` on something already borrowed, a struct field that repeats
`..Default::default()` — delete the redundancy and the mutant stops existing.
Simpler code and a shorter exclusion list, same behaviour.

== Scope

Each bullet removes one `exclude_re` line by refactoring, not by testing:

- `crates/mindtape-cli/src/render/field.rs` `fit_to_width` — `truncate_to_width`
  already returns early when it fits, and `pad_to_width` already returns early
  when it is wide enough, so the `>` branch is dead ceremony. Write it as
  `pad_to_width(&truncate_to_width(content, width), width)`.
- `crates/mindtape-cli/src/render/table.rs` `render_unpadded` — same short-circuit;
  the whole `if display_width(&content) > width` wrapper reduces to
  `truncate_to_width(&content, width)`.
- `crates/mindtape/src/naming.rs` `read_macro` — `chars` is already
  `&mut Chars<'_>`, so drop the no-op `.by_ref()`.
- `crates/mindtape-cli/src/ls_rules.rs` `Applied::match_all` — delete the
  explicit `rule: None`, which `..Self::default()` already supplies.
- `crates/mindtape/src/config_views.rs` `dfs`, ACTIVE arm — the `|| stack.clone()`
  default in `map_or_else` is unreachable because the node is on the stack
  whenever it is ACTIVE. `stack.iter().skip_while(|n| *n != node).cloned().collect()`
  expresses the same thing with no dead default.
- `crates/mindtape/src/numeric.rs` `NumericField::classify_anchor` — the `<`
  scan is only equivalent to `<=` because an exact-match scan ran first. One
  `partition_point` over the already-sorted anchors gives the exact hit and the
  lower neighbour from a single comparison, and then the operator is
  load-bearing. Preserve today's duplicate-value behaviour (exact takes the
  first, lower takes the last).
- `crates/mindtape/src/ready.rs` `Readiness::empty` — the body *is*
  `Self::default()`. Either delete the constructor for `Readiness::default()`
  (~15 call sites, mostly tests) or keep the name and accept the exclusion.
  Judgement call: `empty()` names a domain concept that `default()` does not, so
  this is the one bullet where keeping the exclusion may be the better trade.

Not in scope: `config_views.rs` `dfs`'s `delete match arm DONE`. That arm is
pure memoisation — a node reaches DONE only after its whole subtree came back
cycle-free, so re-walking can only return `None` again. Deleting it costs
exponential time, never correctness, and no amount of restructuring makes a
memo observable. It stays excluded; keep its reason next to it.

== Verification

For each bullet, `cargo mutants -f <file>` *without* `--iterate` after the
refactor, and remove the matching `exclude_re` line in the same commit. A green
suite is not evidence — only the sweep proves the mutant is gone rather than
merely renamed.
