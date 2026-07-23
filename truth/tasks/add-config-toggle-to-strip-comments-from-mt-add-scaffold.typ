#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "add config toggle to strip comments from mt add scaffold",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "mt"),
  estimate: (files: 4),
  status: done(2026, 7, 23),
)

== Summary

Add an \[add\] config switch that renders the interactive YAML scaffold without any \# comment lines (header plus per-slot annotations), for users who want a bare mapping to fill.

== Scope

- Today `[add] header = ""` drops only the top instruction block; the per-slot annotation comments (`# label [kind, required]`, `# choices:`, `# default:`) are always emitted by `push_comment` in `crates/mindtape-cli/src/add_interactive.rs`. No config strips them, so a fully bare scaffold is impossible.
- Add a `comments` bool to `AddConfig` (`crates/mindtape/src/config_add.rs`), default `true`. `false` strips every `#` line from the scaffold — both the header and the per-slot annotations — leaving a bare mapping of slot names to values.
- Thread the flag through `scaffold`/`entry_block`/`header_text`: skip `push_comment` and short-circuit `header_text` to empty when disabled. Keep `header` orthogonal (a custom header still shows only when comments are on).
- Regenerate `schema/config.schema.json` (`MINDTAPE_BLESS=1`) and update the `[add]` docs: `www/content/templates.typ`, `www/content/config.typ`, and this repo's CLAUDE.md add section.
- Tests: `comments = false` yields a scaffold with no `#` line yet still parses back through `parse_buffer`; default stays commented.
