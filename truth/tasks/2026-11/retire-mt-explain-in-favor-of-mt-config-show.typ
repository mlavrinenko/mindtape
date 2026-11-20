#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Retire mt explain in favor of mt config show",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[one command answers what the project is, and a new key cannot fall through],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
  links: depends-on(
    "show-the-resolved-config-in-mt-config-show.typ",
    "add-a-status-mode-to-mt-allow.typ",
    "give-the-dictionaries-their-own-command.typ",
    "give-the-templates-their-own-command.typ",
    "split-contrib-jq-and-add-the-essentials-report.typ",
  ),
)

== AS-IS

+ Run `mt explain` in a project that sets `[add] verify-when`, `[[tasks.roots]] exclude`, `[templates] check-defaults` and `[commits] run`.
+ Ask the index what each does.
+ 🔴 The index shows none of the four, and it is a second presentation of the config that drifts from it:

```
$ mt explain | sed -n '/^commits:/,/^flip:/p'
commits:
  trailer: Refs
  rules (error): name-task-in-refs-trailer, squash-every-autosquash-commit, flip-task-with-its-work,
    finish-named-task-at-tip, keep-tweak-to-one-source-file, add-changelog-entry-for-product-change,
    start-named-task-in-its-commit
add:
  hooks: file-by-month, sweeps
flip:
```

== TO-BE

- 🟢 `mt explain` retires, and `mt config show` is the one command that answers what the project is.
- 🟢 `mt config show --json --resolved` carries every key. A report reads the config rather than a second shape of it.
- 🟢 `mt dicts`, `mt templates` and `mt allow --status` own the three facts the config does not hold.
- 🟢 The moments each rule fires at live in `essentials.jq` until they settle, and the script says so.
- 🟢 An `mt explain` call states the command that replaced it, and no recipe only this repo has.

```
$ mt explain
error: unrecognized subcommand 'explain'
hint: run `mt config show --resolved` for the config, `mt templates` for the templates, or `mt allow --status` for the commands
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/`, `crates/mindtape/src/explain.rs` and the three `explain_*_e2e.rs` tests: deleted, with the seven subcommands, the index renderer and `Shape`.
- `crates/mindtape-cli/src/cli/`: `explain` leaves the command list, and an unknown `explain` names `mt config show --resolved`, `mt templates` and `mt allow --status`. `command_class.rs` drops its seven entries and `tree_json_tests.rs` its case, or the exhaustiveness test fails.
- `schema/output/explain/`, `explain-add/`, `explain-templates/`, `rules/`, `tags/`, `views/`, `view/`: deleted, with their `cmd/schema/registry.rs` entries. `dicts/` and `dict/` stay, owned by `mt dicts` and `mt dicts <name>`.
- `crates/mindtape-cli/src/`, `crates/mindtape/src/`: the 29 surviving files that name `mt explain` in a doc comment, a help string or a test point at the replacement.
- `www/content/cli/explain.typ`: deleted. The four pages that link it lose their anchors: the six "run `mt explain rules`" sentences point at `/cli/config/`, and `cli/add.typ`'s alias at `/cli/templates/`. The eight other pages point at the replacement.
- `www/content/schema.typ`: the paragraph mapping `explain-add`, `explain-templates`, `view` and `dict` to their commands drops the retired ones, and the schema gate recaptures `schema-list.json`.
- `skills/mindtape/SKILL.md`, `skills/mindtape/references/`: step 1 becomes `mt config show --resolved`, step 3's `mt explain templates` becomes `mt templates`, and step 6 retires. The reports stay out.
- `docs/src/agents.typ`, `docs/src/skill.typ`: the same, then `just docs md` regenerates `AGENTS.md` and `SKILL.md`.
- `.mindtape/templates/decision.typ`: its `mt explain rules --for add` line points at `mt config show --resolved`.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Removed`.

== Alternatives

- Keep `mt explain` and fix the four keys: the next config key falls through the same way, because the blocks are hand-built.
- Keep `mt explain` as a thin wrapper over the report: two commands for one answer, and the wrapper still needs the allow join.

== Out of scope

- `mt config show`'s default view. It keeps printing the merged layers, and a Rust-side default stays invisible there. `--resolved` is the view this task points at, and it lands first.
- The reports. `contrib/jq/reports/essentials.jq` replaces the index, and it lands first.

== Dogfood

This repo's `mt explain` prints 56 lines today. After the change, `just orient` prints the same shape from `mt config show --json --resolved`, and the four keys it dropped appear.
