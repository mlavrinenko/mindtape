#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt recur --advance mints one stem for the whole chain",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(
    2026,
    9,
    10,
  )[Both causes fixed. The semantic question is settled the first way: `--advance N` chains the dates off the cursor regardless of the anchor — the first hop honours it (`plan_next`), every hop after steps off the previous occurrence (`plan_step`) — because refusing a non-`due` anchor would refuse `default-anchor = "completion"`, the default, and with it nearly every `--advance` there is. Stems now disambiguate against a `StemDest` carrying the claimed set beside the directory, `mt import`'s shape. Written up in www/content/cli.typ and truth/tasks/recurring-tasks.typ. `just check` green, `cargo mutants --in-diff` 10/10 caught.],
)

== Summary

`mt recur --advance N` has never worked for `N > 1`. Every occurrence in
the chain is minted with the same filename stem, so before the pre-write seam
landed the run printed N `spawned` lines all naming the same path, wrote that
one file N times, and left the last write on disk. It now fails the whole
batch on a duplicate-id violation and writes nothing — honest, but still not
working.

Two independent causes, and the second is the interesting one.

== Scope

- `mint_stem` disambiguates its candidate against files on DISK
  (`dest.join(...).exists()`). No entry of a chain in flight is on disk yet,
  so all N collide. `mt import` already solved this shape: it seeds a
  `HashSet` of claimed stems and disambiguates against that as well as the
  directory (`import.rs`'s `existing_stems`/`stem_for`). `mint_stem`
  is at clippy's 5-argument limit, so the claimed set arrives bundled with
  `dest` rather than as a sixth parameter.
- The dates do not advance either, and a fix for the stems alone would paper
  over it. `build_advance_chain` walks `due_cursor` forward each
  iteration, but a rule anchored on today (the `learn-piano` fixture's
  `every(days: 1)`) resolves every hop to today plus one — so `--advance 3`
  asks for three occurrences and plans three copies of tomorrow. Decide what
  `--advance` means for a today-anchored rule before wiring the stems: chain
  the dates off the cursor regardless of the anchor, or refuse `--advance`
  for a rule whose anchor is not `due`. It is not settled here.
- Regression test: `--advance 3` over a today-anchored daily rule writes
  three distinct files, three distinct dates, each `recurred-from` the one
  before it, and `mt check` passes afterwards.

== Notes

Found while routing the writers through the pre-write seam
(`extend-the-pre-write-validation-seam-to-the-other-writers.typ`). That task
fixed the sibling defect the same walk exposed — the chain's
`recurred-from` edges accumulated instead of replacing, which broke every
hop of the ordinary single-step loop after the first — and stopped there: the
stem collision is a distinct bug whose fix depends on a semantic decision
about `--advance` that the seam has no opinion on.
