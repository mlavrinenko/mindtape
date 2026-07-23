#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "compact the mt add scaffold when comments are off",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli", "mt"),
  estimate: (files: 2),
  status: wip(2026, 7, 23),
)

== Summary

With \[add\] comments = false the scaffold still separates every key: value with a blank line (blocks joined by a newline that mattered only to space out comment groups). Drop the separators when comments are off so the bare mapping is compact.

== Scope

- `scaffold` in `crates/mindtape-cli/src/add_interactive.rs` joins per-slot blocks with `"\n"`. Each block already ends in a newline, so the join inserts a blank line between entries — deliberate spacing between comment+value groups when comments are on, but noise once comments are stripped.
- Join with `""` (no separator) when `config.comments` is false, keeping `"\n"` when on. Bare mapping becomes consecutive `key: value` lines.
- Test: `comments = false` scaffold has no blank line between entries; `comments = true` still does.
