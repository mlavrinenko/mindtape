#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rename the Rule packs page to Packs",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 7),
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
      sound-codebase: neutral,
      docs-match-code: helps[the page's name matches packs that carry every config key, not just rules],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "let-a-pack-carry-any-config-key-templates-and-typst-files.typ",
    ),
)

== Problem

+ Open `www/content/rule-packs.typ`, served at `/rule-packs/`.
+ 🔴 Once a pack carries a legend, a priority and templates, the page's name and route say it holds rules only, and nine pages link it under that name.


== TO-BE

- 🟢 The page is `www/content/packs.typ`, titled "Packs", at `/packs/`, and every link, slice, capture and outdatty entry names it there.
- 🟢 Its content is unchanged: the leaves of `share-config-between-projects-through-packs.typ` rewrite it section by section.


== Source of truth

`crates/mindtape/src/config/packs.rs` and `crates/mindtape/src/config/pack_resolve.rs`, which the page describes.


== Changes

- `www/content/rule-packs.typ`: move to `www/content/packs.typ`, retitled, with `route: "/packs/"`.
- `www/content/*.typ`, `www/content/cli/rules.typ`: point every `/rule-packs/` link at `/packs/`.
- `www/utils/site.typ`: the nav entry.
- `www/generate-rule-packs.sh`: move to `www/generate-packs.sh`, its captures renamed `packs-*`, and `Justfile`: call it there.
- `www/generate-examples.sh`: the `CONFIG_FACTS` entry and the comment.
- `.mindtape/config.toml`: the `docs:slice packs/...` names.
- `.config/outdatty.yaml`, `.config/outdatty.lock`: the two entries, re-recorded.


== Out of scope

- Any change to what the page says: each leaf of the milestone rewrites its own section.


== Dogfood

This repo's docs build serves `/packs/`, and `just check` passes with no `/rule-packs/` left.

