#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fold the four policy listings into mt explain",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[one command answers what the project enforces and where each entry comes from],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[one CLI page and one listing surface replace four pages that restate each other],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
  links: depends-on(
    "decide-on-one-command-for-the-projects-policy-listings.typ",
  ),
)

== AS-IS

+ Run `mt rules` in this repo.
+ 🔴 It prints every rule with its message, while the tags, views and dictionaries sit behind three other commands.
+ Run `mt tags`, `mt views` and `mt dicts`.
+ 🔴 Four entry points answer one question, each under its own `--json` envelope, and none names the other three.
+ Start a session without running any of them.
+ 🔴 An agent has no cheap index of the rules, tags, views and dictionaries this project uses.


== TO-BE

- 🟢 `mt explain` is the one command. Bare prints a config-only index: rules grouped by section, derived fields, tags, views, and dictionaries with their entry counts.
- 🟢 `mt explain rules|tags|views|dicts` prints one listing unchanged; `--for add|flip|commit` stays on rules and tags.
- 🟢 `mt explain --json` prints the index; each listing keeps its JSON keys and envelope.
- 🟢 The four old commands are removed whole, with no aliases.

```text
$ mt explain
rules:
  tasks.rules: statefulness-tag, imperative-title, ...
  tasks.body.headers.rules: section-allowlist, ...
  tasks.body.content.rules: as-is-is-numbered-steps, ...
  commits.rules: work-names-a-task, ...
  on.flip: stale-wip, gates-fresh
derived fields: changes
tags: mt, substrate, ...
views: default, agent, ask-refactoring, blocked, owner
dicts: product (4), source (16), ...

mt explain <rules|tags|views|dicts>
```


== Changes

- `crates/mindtape-cli/src/cmd/explain/mod.rs` (new): `ExplainArgs`, global `--json`, `Listing`, dispatch.
- `crates/mindtape-cli/src/cmd/explain/index.rs` (new): the grouped config-only index, its JSON, default first and wrapping.
- `crates/mindtape-cli/src/cmd/explain/rules.rs` (new): the rules body, renderer and tests moved from `cmd/rules.rs`.
- `crates/mindtape-cli/src/cmd/explain/tags.rs` (new): the tag scan and tests from `cmd/tags.rs`.
- `crates/mindtape-cli/src/cmd/explain/views.rs` (new): the views body and detail renderer from `cmd/views.rs`.
- `crates/mindtape-cli/src/cmd/explain/dicts.rs` (new): the dictionary list and entry detail from `cmd/dicts.rs`.
- `crates/mindtape-cli/src/cli/mod.rs`, `crates/mindtape-cli/src/cli/command_class.rs`, `crates/mindtape-cli/src/cmd/mod.rs`: replace the four with `Explain`; classify it and its listings as `Read::One`.
- `crates/mindtape-cli/src/cli/tree_json.rs`, `crates/mindtape-cli/src/cli/tree_json_tests.rs`, `crates/mindtape-cli/tests/it/read_no_write_e2e.rs`: a runnable parent carries its class and reach.
- `crates/mindtape-cli/src/cli/completion/mod.rs`, `crates/mindtape-cli/src/cli/completion/tests.rs`, `crates/mindtape-cli/tests/it/completion_e2e.rs`: the view and dict sources move under `explain`.
- `crates/mindtape-cli/src/util.rs`, `crates/mindtape-cli/src/util_tests.rs`, `crates/mindtape-cli/src/cmd/ls/rules_tests.rs`, `crates/mindtape-cli/src/diag/from_lib.rs`, `crates/mindtape-cli/src/diag/tests_dict.rs`: hints name `mt explain views` and `mt explain dicts`.
- `crates/mindtape-cli/tests/it/explain_e2e.rs` (new): red tests for the index and four listings; replace the four e2e files and update `tests/it/main.rs` and `tests/it/derived_fields_e2e.rs`.
- `www/content/cli/explain.typ` (new): the page; delete the four pages, update `www/content/cli.typ` and `www/utils/site.typ`, and move the notes to `explain.rules` through `explain.dicts`.
- `www/generate-cli.sh`, `www/generate-reference.sh`: captures run `mt explain rules`, `mt explain views` and `mt explain dicts`.
- `www/content/named-sets.typ`, `www/content/agents.typ`, `www/content/config.typ`, `www/content/rules.typ`, `www/content/require-a-task-shape.typ`, `www/content/start-titles-with-a-verb.typ`: prose names `mt explain` and its listings.
- `docs/src/skill.typ`, `docs/src/agents.typ`, `skills/mindtape/references/filters.md`, `.mindtape/templates/decision.typ`: say run `mt explain` first; regenerate `SKILL.md` and `AGENTS.md`.
- `crates/mindtape/src/config/views.rs`, `crates/mindtape/src/config/hooks.rs` and the other config modules: schema descriptions say `mt explain rules` and `mt explain views`.
- `CHANGELOG.md`: an `[Unreleased] Changed` entry naming the removal and the command.


== Alternatives

- Counts only at bare: the names are the index, and a name says more than a count.
- Keep the four commands and add `mt explain` as a wrapper: two names per listing, and the docs still list four entry points.
- Absorb only rules and tags: views and dicts answer the same project-policy question, so a reader still runs two commands.
- Require a listing after `mt explain`: one more word on every read, and no place that teaches a reader the four listings exist.
- A deeper tree, `mt explain rules body`: more verbs than the four listings need.


== Out of scope

- New listing content or extra fields; the four detail listings keep what they print today.
- A config key or JSON schema change; the four keep their envelopes, and only the new index gets a `schema_version`.
- Origin and readers on each index entry; `mt config show --origin` stays the source for origins.
- Renaming the rules; `make-every-rule-name-say-what-it-checks.typ` reviews the names the index shows.


== Dogfood

This repo runs `mt explain` first in a session, `mt explain rules --for flip` and `mt explain tags --for add` before a flip or a filing, and `mt explain views` before `mt ls @name`. The skill and AGENTS.md name one listing surface.
