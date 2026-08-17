#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "relativize the crap baseline's committed file paths",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 13),
  links: related("scan-the-tree-for-secrets-with-gitleaks.typ")[
    the secret scanner that cannot take this job
  ],
  status: done(
    2026,
    8,
    17,
  )[1056/1058 entries pair through the suffix pass; gate green, ratchet still fires],
)

== Summary

`crap-baseline.json` carried an absolute `file` for all 1048 entries — `/home/<user>/projects/home/mindtape/crates/...` — because `cargo crap --workspace` walks the member roots `cargo metadata` reports and cargo-crap 0.3.1 has no relativize flag. That publishes the minter's own machine layout in a tracked file, and it buys nothing: cargo-crap keys its first matching pass on `(file, function, line)`, so every entry already missed that pass under CI's `/home/runner/work/mindtape/mindtape` checkout and paired through the suffix-aware pass instead. The exact match held on exactly one machine — the one that minted the file.

== Scope

- `crap-baseline.sh` strips the workspace root (`cargo metadata`, not `$PWD` — it is the root cargo-crap derived the paths from, so a symlinked checkout still matches) from every entry after minting
- a `crap-baseline-check` arm fails on an absolute, UNC, drive-lettered or `..`-bearing entry, and on an entry-less baseline, which would pass the ratchet by reporting every function as new. It carries a positive control over a known-bad fixture, because a filter matching nothing reads exactly like a clean file
- rationale into `docs/src/contributing/coverage-crap.typ`, rendered to the deep dive
- out of scope: an upstream `--relative-paths` flag on cargo-crap, which would restore exact matching on both sides

== Measured

The first relative mint pairs 1056 of 1058 entries as `unchanged` against an absolute current report. The two that do not are one file's two same-named `fake_editor` functions (`add_interactive_tests_editor.rs`), ambiguous by suffix and therefore already unpaired under CI's root. A baseline entry doctored downward still fails the gate naming the function (`unlink_edits`, +5.0), so the ratchet reads deltas exactly as before.

The mint also absorbed two regressions that had sat un-minted since the baseline was last written on 2026-08-09: `command_json` (`cli_dump.rs`, CRAP 1.0 → 2.0, filed by 5b3e53f) and `build_key` (`cache.rs`, 2.0 → 3.0, by 9fccd82). Both are one extra branch at 100% coverage. `just crap-baseline` refuses to mint over a red gate, so the mint ran through its script directly — the recovery path the deep dive documents.
