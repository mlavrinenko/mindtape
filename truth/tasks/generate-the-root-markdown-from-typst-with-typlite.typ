#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "generate the root markdown from typst with typlite",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 3,
  ),
  difficulty: 6,
  tags: ("docs", "tooling"),
  estimate: (files: 12),
  links: depends-on(
    "trim-root-docs-by-delegating-deep-content.typ",
    "derive-the-drift-prone-doc-facts-from-their-sources.typ",
  ),
  status: wip(2026, 8, 13),
)

== Summary

Author README.md, AGENTS.md, CONTRIBUTING.md, SKILL.md and docs/contributing/\*.md as Typst, render them to Markdown with typlite, and gate freshness. One source then feeds both the site and the repo-root file, and the generated docs can read www/generated instead of restating it.

== Scope

- tinymist into flake.nix devShell: nixpkgs ships the typlite binary with it (96 MB / 244 MB unpacked, cached). Verify the packaged typlite carries typst 0.14 — tinymist 0.14.20 pins 0.14.2, but nixpkgs unstable has moved to 0.15.x
- Decide where the .typ sources live and how a body is shared with www/content (today every page opens with \#show: page.with(...) plus \@tola imports, neither of which typlite should see)
- Freshness gate: typlite --check plus git status --porcelain, the tinymist pattern; wire into just check
- Move the linecop cap onto the .typ source, since an oversize in generated output cannot be edited away. linecop supports Typst; note that www/content/config.typ is 747 lines and trips any cap immediately
- SKILL.md needs its YAML frontmatter from a typst query on a \#metadata block, prepended to the typlite body — typlite cannot emit frontmatter
- Read-only banner at the top of every generated file naming its source
- CHANGELOG.md stays hand-written: it is history, it does not drift from code
