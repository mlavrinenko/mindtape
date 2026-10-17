#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render the built-in link kinds from BUILTIN_KINDS",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("docs", "www", "cli"),
  estimate: (src-files: 5),
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
      docs-match-code: helps[the links page and `mt link --help` read the kinds off the table `mt check` resolves them with],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "render-the-filter-field-table-from-the-field-registry.typ",
    )[both add a registry to tree\_json.rs and generate-facts.sh, so landing in turn avoids a conflict in the same lines],
)

== Problem

+ Read "The built-in kinds" in `www/content/links.typ`, then run `mt link --help`.
+ 🔴 The page types one row per kind and says "next to the six built-ins". The help types "The built-in kinds are `depends-on`, `parent`, `recurred-from`, `supersedes`, `duplicates` and `related`". `BUILTIN_KINDS` in `crates/mindtape/src/link/mod.rs` is in no outdatty group, so a seventh kind leaves all three green.


== TO-BE

- 🟢 `mt _cli --json` carries `link_kinds`: each built-in's name, `max`, `blocking` and `acyclic`, read off `BUILTIN_KINDS`.
- 🟢 The links page renders its table and its count from that fact and `www/utils/link-kind-notes.typ`, and the build fails on a kind with no note.
- 🟢 `mt link --help` lists the kinds from the same table.


== Source of truth

- `crates/mindtape/src/link/mod.rs`: `BUILTIN_KINDS` and each kind's rule.


== Changes

- `crates/mindtape-cli/src/cli/tree_json_tests.rs`: the red test first: `link_kinds` holds every built-in with its rule, and the `KIND` help of `mt link` names each.
- `crates/mindtape/src/link/mod.rs`: a public accessor over `BUILTIN_KINDS`.
- `crates/mindtape-cli/src/cli/tree_json.rs`: export `link_kinds`.
- `crates/mindtape-cli/src/cmd/link/mod.rs`: build the `KIND` help from the accessor.
- `www/generate-facts.sh`: write `link-kinds.json` from `cli.json`.
- `www/utils/link-kind-notes.typ`: what each edge means.
- `www/content/links.typ`: render the table and the count from the fact.


== Out of scope

- Custom kinds from `[[tasks.links.kinds]]`, which the page shows from the schema and a fixture.


== Dogfood

This repo's tasks use `depends-on`, `parent` and `related` daily, and its docs build fails on a kind change until the note follows.

