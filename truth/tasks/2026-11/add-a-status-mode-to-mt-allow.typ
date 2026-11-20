#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add a status mode to mt allow",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
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
      self-describing: helps[a reader sees which commands the config runs before trusting them],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Declare an `[[on.rm]]` action that runs a command.
+ Ask which commands the config declares, before deciding to trust them.
+ 🔴 The only command that lists them also records the approval, so inspecting the list grants it:

```
$ mt allow
allowed in this repository and its worktrees:
  scripts/archive
```

== TO-BE

- 🟢 `mt allow --status` lists every command the config declares, under one status line for the set, and doesn't record anything.
- 🟢 `mt allow` keeps recording the approval, and prints the same table under an `allowed` line.
- 🟢 `--json` carries the same list, so a script can join it with `mt config show --json --resolved`.
- 🟢 One enumerator feeds both modes, so the listing and the record cannot disagree.
- 🟢 Both modes print the same table, so the record and the listing read alike.

```
$ mt allow --status
not allowed in this repository and its worktrees:
  on.flip  mmz --is-fresh --tag gate
  on.rm    scripts/archive
```

== Changes

- `crates/mindtape/src/allow.rs`: `gate_commands` becomes `sites`, returning the kind (gate or editor), the moment and hook label when it has one, and the command, so one walk feeds every caller. `commands` derives from it, so the `[go]` and `[add]` editors `mt allow` records appear in `--status` too. `refused_gates` keeps filtering to gates, so the warning that points at `mt allow` never names an editor.
- `crates/mindtape-cli/src/cmd/allow.rs`: `--status` prints the sites under the set's one status line and doesn't record anything. `--json` emits the same list.
- `crates/mindtape-cli/src/cmd/allow_tests.rs`: `--status` leaves the store untouched, and a config with no commands prints the same `nothing to allow` line bare `mt allow` prints.
- `crates/mindtape-cli/tests/it/allow_e2e.rs`: the end-to-end listing and the JSON shape.
- `schema/output/allow-status/1.schema.json`: the new output.
- `www/content/cli/allow.typ`: the read mode, and the help text that omits `[[on.rm]]`.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`, and one under `Changed` for `mt allow`'s output gaining the gate label.

== Alternatives

- Make bare `mt allow` print the full list and add `--yes` to record: it flips the safe default. A reader who runs the command to look also grants.
- A separate `mt allow list` subcommand: `mt allow` has no subcommands today, and a flag keeps the one command.

== Out of scope

- The store's format and location. `--status` reads what `mt allow` writes.
- The `not allowed` label `mt explain` prints beside a gate. It retires with `mt explain`, and `--status` prints its own header.

== Dogfood

This repo allows one command, `mmz --is-fresh --tag gate`, so `mt allow --status` prints one `allowed` header and one row.
