#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the user docs generated or gated, not restated",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  kind: "milestone",
  tags: ("docs", "www"),
  estimate: (src-files: 0),
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
      docs-match-code: helps[every user doc fact a registry holds comes from a generator or fails a gate once the two differ],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
)

== Problem

+ Change one line of `crates/mindtape-cli/src/cmd/ls/mod.rs`, as `dc34aced` did, and run `just outdatty-check`.
+ 🔴 It prints `[ stale ]  cli-docs` and 27 `review dependent:` lines, one of them a glob over 25 command pages. Four commits on 16 and 17 October re-recorded that group with no doc edit.
+ Look up the board's keys, the built-in link kinds or the template slot kinds on the site.
+ 🔴 Each is typed by hand beside a registry no gate reads. A bad slot kind in a template already makes `mt add` print `expected one of string, int, float, real, bool, date, list, text, raw, choice`, without the `multichoice` the templates page lists.


== TO-BE

- 🟢 A user doc states a fact a registry holds only through a generator, or under a gate that fails when the two differ.
- 🟢 An outdatty group fires only for the prose its source feeds, so a re-record means someone reviewed that prose.
- 🟢 `mt --help`, the quickstart and the sidebar lead to `mt @name`.
- 🟢 The site's voice holds in generated text too: no schema description or help line carries an em dash.


== Changes

- `truth/tasks/*.typ`: the child tasks, linked with `parent` to this one. This milestone changes no file itself.


== Out of scope

- New `mt` features beyond the registries the docs read.
- AGENTS.md and the skill beyond the facts a child derives. An agent doc keeps its own copy by design.


== Dogfood

This repo's `just check` runs every new gate, and its own commits carry the trailer the outdatty hook asks for.

