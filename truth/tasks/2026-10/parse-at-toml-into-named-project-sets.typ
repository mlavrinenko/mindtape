#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Parse at.toml into named project sets",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[one TOML file lists every project and set, with no registry to run],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("show-every-tracked-project-from-one-mt-name.typ"),
)

== AS-IS

+ Put `[[worktrees]]` entries with `name`, `path` or `base` in `~/.config/mindtape/worktrees.toml`.
+ Run `mt worktrees`.
+ 🔴 Each entry lists one project: none can list the 30 projects under `~/projects/home` as a set, skip a fork, or join two sets.


== TO-BE

- 🟢 `mindtape::at` parses `at.toml`. Each `[at.<name>]` has `include` (paths, one-level globs or `@name`s), `exclude`, and `forks`: `false` (the default), `true` or `"only"`.
- 🟢 `@x` is `[at.x]`, else the directory named `x` that an `include` matches, whatever its `forks`. mt refuses two such directories and lists both. So is an `@name` cycle.
- 🟢 An entry whose `include` is one literal path is a project. Any other entry is a set, even when it matches one project, so output follows the config, not the disk.
- 🟢 A fork is a linked git worktree: its gitdir holds `commondir`, which the gitdir of a submodule lacks.
- 🟢 A member takes its name from its directory, or by its shortest unique path suffix on a clash.


== Changes

- `crates/mindtape/src/at/tests.rs`: the red tests first: glob members, `exclude`, each `forks` mode, name lookup, the clash and cycle refusals, a one-path entry as a project.
- `crates/mindtape/src/at/mod.rs`: the entry types, parsing with `deny_unknown_fields`, and the `//!` file map.
- `crates/mindtape/src/at/resolve.rs`: `@x` to a project or a set, `@name` expansion and `exclude`.
- `crates/mindtape/src/at/scan.rs`: glob expansion with `globset`, the `.mindtape/` test and fork detection, behind an injected filesystem so tests never touch disk.
- `crates/mindtape/src/lib.rs`: re-export the module.


== Alternatives

- Keep `[[worktrees]]` and add `[[sets]]`: two entry kinds answering one question, which directories `@x` means.
- `**` globs: a home directory walk on every `mt @x`. One level per `*` reads only the listed directories.
- `forks = true` by default: a fork repeats its main checkout's tasks.Every count doubles.


== Out of scope

- The CLI side, `mt at` and the `@x` rewrite: the next task switches it.
- `[board]`, which the board columns task parses.
- `worktrees.toml`, which stays until the CLI switches.


== Dogfood

The `just fork` worktrees beside this repo stay out of a set unless its `forks` asks for them, and `@mindtape` still finds this repo by its directory name.

