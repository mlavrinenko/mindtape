#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show paths in the native separator, with a --path-separator override",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli", "mt"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a Windows user can paste every path `mt` prints into cmd or PowerShell as is],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one helper renders each kind of path, so a Linux test pins what Windows prints],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ On Windows, run `mt check` in a project whose template breaks a rule.
+ 🔴 It prints `error: .mindtape/templates\bug.typ:8: ...`, with both separators in one path.
+ Run `mt ls --json`.
+ 🔴 The `file` field reads `tasks\\a.typ`, so a script written on Linux misreads it.


== TO-BE

```console
> mt ls --format path
tasks\a.typ
> mt --path-separator / ls --format path
tasks/a.typ
> mt ls --json
[{"file": "tasks/a.typ", ...}]
```
- 🟢 A path shown to people uses the native separator: the `file` column, `--format path`, error places and command messages.
- 🟢 A path read by programs uses `/`: the JSON and CSV `file` field, the JSON `place`, and any stored value.
- 🟢 The global `--path-separator <SEP>` sets the separator of every path shown to people, as ripgrep and fd do, and leaves program output alone.


== Changes

- `crates/mindtape/src/path_text.rs` (new), `path_text_tests.rs`: the red tests first, with an injected separator; `display_path`, `slash_path` moved here, and `init_path_separator`.
- `crates/mindtape/src/lib.rs`: export them.
- `crates/mindtape-cli/src/cli/mod.rs`: the global `--path-separator` flag, one ASCII character.
- `crates/mindtape-cli/src/main.rs`: set the separator before any command runs.
- `crates/mindtape-cli/src/render/file_path.rs`: the `file` column and `--format path` go through `display_path`.
- `www/content/cli.typ`, `docs/src/contributing/windows.typ`: the rule and the flag.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- `/` everywhere: a Windows user can't paste a path into cmd, which reads `/` as a switch.
- Native in JSON too, as cargo does: a script then parses the output differently on each OS, and a stored record means different things.
- A `--path-separator` on each command: `--color` is global because every command prints errors, and errors carry paths too.


== Out of scope

- Routing every other printed path through the helper: `route-every-path-mt-prints-through-one-helper.typ`.
- The hyperlink URIs: `write-the-file-columns-hyperlink-as-an-rfc-8089-file-uri.typ`.


== Dogfood

Not applicable: this repo is developed on Linux, where the native separator is `/` already.

