#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve mt @name through at.toml and rename mt worktrees to mt at",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 9),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[one file, `at.toml`, configures every `@name`],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
  links: parent("show-every-tracked-project-from-one-mt-name.typ")
    + depends-on("parse-at-toml-into-named-project-sets.typ"),
)

== Problem

+ Write `[at.home]` in `~/.config/mindtape/at.toml`.
+ Run `mt @home ls`, then `mt at`.
+ 🔴 `mt` reads only `worktrees.toml`: the first prints `error: unknown worktree: '@home'`, and `at` is not a command.


== TO-BE

- 🟢 A leading `@x` resolves through `at.toml`. A project becomes `-w <path>` as today, so `mt @sn ls` is unchanged.
- 🟢 A set is held for the commands that call `util::open_set`, and `util::open` refuses it:
```
error: `flip` runs on one project, and @home holds 27
hint: name one project, such as mt @speconaut flip
```
- 🟢 `mt at` lists the entries, `mt at home` lists its members with their paths and marks forks, and `--json` prints either.
- 🟢 `worktrees.toml`, `mt worktrees` and `mindtape::WorktreeRegistry` are gone.


== Changes

- `crates/mindtape-cli/src/cmd/at_tests.rs`: the red tests first, from `worktrees_tests.rs`: a project rewrites to `-w`, a set is held and refused, `mt at home` lists members.
- `crates/mindtape-cli/src/cmd/at.rs`: from `cmd/worktrees.rs`, reading `at.toml` through `mindtape::at`.
- `crates/mindtape-cli/src/util.rs`: the held set, `open_set`'s stub, and the refusal in `open`.
- `crates/mindtape-cli/src/cli/mod.rs`, `crates/mindtape-cli/src/cmd/mod.rs`, `crates/mindtape-cli/src/cli/command_class.rs`, `crates/mindtape-cli/src/cli/check_argv.rs`, `crates/mindtape-cli/src/main.rs`: the rename.
- `crates/mindtape/src/worktrees.rs`, `crates/mindtape/src/lib.rs`, `crates/mindtape/src/loader/app_dirs.rs`: drop the old registry and its mentions.
- `www/content/cli/at.typ`, `www/content/agents.typ`, `www/content/config.typ`, `docs/src/skill.typ`, `outdatty.yaml`: name `at.toml` and `mt at`.


== Alternatives

- Keep `mt worktrees` as an alias: there's no compatibility to keep before the first release.
- Run a set's command once per member until each command learns sets: `mt @home flip` would write in 27 projects.
- An allowlist of set commands in `cli/mod.rs`: every command task would edit it, while calling `open_set` is the opt-in.


== Out of scope

- Running a command on a set: `ls`, `find`, `check` and `board` each learn it in their own task.
- Naming the project that holds a task in the refusal: it needs every member loaded for a hint.
- `-w` and `MINDTAPE_WORKTREE`, which stay one project.


== Dogfood

The owner's `worktrees.toml` becomes an `at.toml` with `[at.home]`, and `mt @mt` keeps reaching this repo.

