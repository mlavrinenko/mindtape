#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Select content-rule sections through artifact-gate and typst-cst",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("mt",),
  estimate: (src-files: 3),
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
      sound-codebase: helps[the content gate stops carrying copies of artifact-gate's path traversal and typst-cst's section span],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 28),
)

== Problem

+ Read `match_path`, `all_descendants`, `seg_matches` and `is_recursive` in `crates/mindtape/src/body/content_gate/mod.rs`.
+ 🔴 They copy artifact-gate 0.1.0's private path traversal line for line, over a `CompiledSeg` enum that mirrors `artifact_gate::Matcher`, because artifact-gate exposed no path selection.
+ Read `section_content` in the same file.
+ 🔴 It recomputes a section's byte span from the flat heading list, because typst-cst 0.2.0 gave a heading's own span only.


== TO-BE

- 🟢 The content gate compiles its paths with the header gate's `compile_path` and selects sections with `artifact_gate::select_by` (artifact-gate 0.2.0) over the `typst_cst` heading tree.
- 🟢 It slices each section by `Heading::section` (typst-cst 0.3.0).
- 🟢 `mt check` reports exactly what it did before: the existing content-gate tests pass unchanged.


== Changes

- `crates/mindtape/Cargo.toml`, `Cargo.lock`: bump `artifact-gate` to 0.2.0 and `typst-cst` to 0.3.0.
- `crates/mindtape/src/body/content_gate/mod.rs`: drop `CompiledSeg`, its compiler, the traversal and `section_content`; select and slice through the crates. No red test, since behavior holds: the existing tests guard it.
- `crates/mindtape/src/body/gate.rs`: share `compile_path` with the content gate.


== Out of scope

- The `no-subsections` rule in `crates/mindtape/src/packs/tasks.toml`. Its `forbid` over a wildcard pair states the rule directly, and the empty-allowlist respelling was cancelled in `adopt-the-empty-allowlist-from-artifact-gate-0-1-1.typ`.


== Dogfood

This repo's own `mt check` runs every content rule in its packs through the shared selection.

