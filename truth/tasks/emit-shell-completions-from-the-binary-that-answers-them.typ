#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Emit shell completions from the binary that answers them",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("cli", "mt"),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a new user reaches the project's own stems and statuses without reading the config first],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[nothing is generated and committed, so nothing can go stale],
      docs-match-code: helps[a completion arm has to name its value source or fail a gate],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 22),
)

== Problem

+ Press tab after `mt flip ` in any shell.
+ 🔴 Nothing completes. `mt install` writes the Typst package, the XDG
  `defaults.toml` and the JSON Schema, and stops there.
+ The values worth completing are all project-derived: task stems, the
  legend's verbs, view names, template stems, link kinds and tags. A
  committed completion script would state them once and be wrong for the
  next project, which is why none was ever written.


== TO-BE

- 🟢 A shell hook installed by `mt install` calls the binary back, so the
  static half is clap's own parser and the dynamic half is the live config.
  Nothing is generated into the repo, so nothing can drift from it.
- 🟢 Tab after `mt flip ` offers this project's legend, not a baked list;
  after `mt ls @` it offers this project's views; after `mt add -t` its
  templates.
- 🟢 A completion over a project that fails to load prints no candidates and
  no error, because a broken tab-press must not look like a broken shell.
- 🟢 A new subcommand with a completable argument fails a gate until it says
  where its values come from.


== Changes

- `crates/mindtape-cli/Cargo.toml` — `clap_complete` with its dynamic engine.
- `crates/mindtape-cli/src/completion.rs` — value sources per argument,
  resolved against the loaded project, each falling back to no candidates
  when loading fails. The red test comes first: a completion over
  `examples/minimal` offers that legend and not `learn-piano`'s.
- `crates/mindtape-cli/src/cli.rs` — attach a completer to every argument
  whose values the project defines.
- `crates/mindtape-cli/src/install.rs` — write the shell hook, behind its own
  `--no-completions` skip, matching the three installers already there.
- `.just/scripts/check-doc-facts.sh` — an arm set-differencing the arguments
  clap declares against the ones `completion.rs` gives a source, so a new
  completable argument cannot ship silently uncompleted.


== Alternatives

- `clap_complete::generate` writing four committed scripts. Rejected for
  exactly the reason this was never done: a static script cannot name a stem
  or a status, which is the half that matters.
- A hand-written bash function shelling out to `mt ls --format path`.
  Rejected because it duplicates the parser, and drifts the first time a flag
  is renamed.


== Out of scope

- Completing a value that needs a full task evaluation, such as a filter
  expression. The filter language deserves its own decision.
- Packaging the hook for a distribution. `mt install` owns the user
  environment here, as it already does for the prelude and the schema.


== Dogfood

This repo is the first project to install it. Tab after `mt flip ` must
offer the six statuses in `.mindtape/config.toml`, and tab after `mt ls @`
must offer the views declared there, neither of which any static script
could know.

