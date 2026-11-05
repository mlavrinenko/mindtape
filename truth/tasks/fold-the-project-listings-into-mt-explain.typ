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
  difficulty: 3,
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
+ 🔴 It lists every rule, and the tags, views and dictionaries those rules read live behind three other commands.
+ Run `mt tags`, `mt views --json` and `mt dicts`.
+ 🔴 Each prints one slice of the same project policy under its own `--json` envelope, and none names the other three.


== TO-BE

- 🟢 `mt explain` is the one command: bare prints an overview of the four listings and their counts, and `mt explain rules|tags|views|dicts` prints one listing.
- 🟢 `--for add|flip|commit` stays on `mt explain rules` and `mt explain tags`, and every `--json` keeps its current keys and envelope.
- 🟢 `mt rules`, `mt tags`, `mt views` and `mt dicts` are removed whole, with no aliases, so the docs and the skill name one surface.

```
$ mt explain
rules  every task, body and commit rule
tags   the allowed tags and the rules that read them
views  the saved views
dicts  the named dictionaries

$ mt explain rules --for flip --json
{"schema_version": 1, "rules": [...], "derived": [...]}
```


== Changes

- `crates/mindtape-cli/src/cmd/explain/mod.rs` (new): `ExplainArgs`, the optional `Listing` subcommand, the bare overview and the dispatch.
- `crates/mindtape-cli/src/cmd/explain/rules.rs` (new): the rules body, renderer and `_tests.rs` moved; `tags.rs` beside it with the tag scan and its tests.
- `crates/mindtape-cli/src/cmd/explain/views.rs` (new): the views body and detail renderer; `dicts.rs` beside it with the list and entry detail.
- `crates/mindtape-cli/src/cli/mod.rs` and `src/cli/command_class.rs` and `src/cmd/mod.rs`: replace the four variants and modules with `Explain`, and classify `explain rules` through `explain dicts` as `Read::One`.
- `crates/mindtape-cli/tests/it/explain_e2e.rs` (new): one red test per listing with its current `--json` envelope, plus the bare overview; replace `rules_e2e.rs`, `rules_list_e2e.rs`, `views_e2e.rs` and `dicts_e2e.rs` and update `tests/it/main.rs`.
- `www/content/cli/explain.typ` (new): the page and its prose; delete `www/content/cli/{rules,tags,views,dicts}.typ`, repoint the links in `www/content/{rules,packs,hooks}.typ`, and replace the four `www/utils/cli-notes.typ` keys with one.
- `docs/src/{skill,agents}.typ`: the command table and the "never guess" paragraph name `mt explain`; then run `just docs md` and `just docs-gen`.
- `CHANGELOG.md`: an `[Unreleased] Changed` entry naming the removal and the one command.


== Alternatives

- Keep the four commands and add `mt explain` as a wrapper: two names per listing, and the docs still list four entry points.
- Absorb only rules and tags: views and dicts answer the same project-policy question, so a reader still runs two commands.
- Require a listing after `mt explain`: one more word on every read, and no place that teaches a reader the four listings exist.
- A deeper tree, `mt explain rules body`: more verbs than the four listings need.


== Out of scope

- New listing content or extra fields; the four keep what they print today.
- A config key or JSON schema change: the listings read the merged config, and each `--json` keeps its keys.
- A richer entry with origin and readers; `mt config show --origin` stays the source for origins.


== Dogfood

This repo runs `mt explain rules --for flip` and `mt explain tags --for add` before a flip or a filing, and `mt explain views` before `mt ls @name`. The skill, AGENTS.md and the CLI docs name one listing surface.

