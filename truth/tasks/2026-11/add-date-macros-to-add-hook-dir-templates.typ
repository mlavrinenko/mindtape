#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add date macros to add-hook dir templates",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "tests", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a dated folder states the filing date without opening git],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a dated layout doesn't need a project helper],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the hook docs and the config token table share one grammar],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 13),
)

== AS-IS

+ Give an `[[on.add]]` hook a dated destination, `actions = [{ dir = "tasks/{date:YYYY}-{date:MM}" }]`.
+ 🔴 `mt add` refuses the whole config: `on.add[0].actions[0].dir: unknown dir macro: {date:YYYY}`.
+ A dated file stem already works under `[tasks.naming] stem`, but a dated folder needs a Typst `derive` function, and `typst-world` doesn't expose today to one.

```toml
[[on.add]]
name = "by-month"
actions = [{ dir = "tasks/{date:YYYY}-{date:MM}" }]
```

```
$ mt add "File the task"
error: .mindtape/config.toml:6: on.add[0].actions[0].dir: unknown dir macro: {date:YYYY}
```


== TO-BE

- 🟢 A static `dir` understands `{date}`, `{date:FMT}`, `{yy}`, `{mm}` and `{dd}`, with the same token grammar as `[tasks.naming] stem`.
- 🟢 `{date:YYYY}-{date:MM}` expands to one path component from the pinned or system date, and `mt add` creates the folder.
- 🟢 The parser still refuses an unknown macro or a format that expands to a separator before it writes anything.

```toml
[[on.add]]
name = "by-month"
actions = [{ dir = "tasks/{date:YYYY}-{date:MM}" }]
```

```
$ MINDTAPE_NOW=2026-11-13 mt add "File the task"
wrote tasks/2026-11/file-the-task.typ (abc)
```


== Changes

- `crates/mindtape/src/config/add_hooks_dir.rs`: accept the date macro family in a static `dir`, and expand each one from a given date through `format_iso`'s tokens.
- `crates/mindtape/src/config/add_hooks_dir_tests.rs`: the accepted family, the refused near miss, the parsed macro names and each expansion.
- `crates/mindtape/src/config/add_hooks.rs`: re-export the date macro helpers and name them in the `dir` docs.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`: expand each date macro from `today_ymd()`.
- `crates/mindtape-cli/src/cmd/add/hooks_tests.rs`: a date macro reads the process date, not the draft.
- `crates/mindtape-cli/tests/it/add_hooks_e2e.rs`: a pinned clock lands a task in the month folder, and the command refuses a separator in a format.
- `crates/mindtape-cli/tests/it/add_hooks_support.rs`: an `add` that pins `MINDTAPE_NOW`.
- `www/content/hooks.typ`: document the macros beside `{template}` and the field macros.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A project `derive` function: it cannot read today from `typst-world`, and every project repeats the same helper.
- A new `{filed}` task field: a model change where a folder is enough, and existing tasks have no such date.
- `strftime` tokens such as `%Y-%m`: they disagree with the naming grammar, where `%` already passes through literally.


== Out of scope

- Date macros in any other config value stay out. Only `dir` takes them.
- A `filed` task field or a backfill of filing dates.
- `mt import` and `mt recur` placement, which keep their own paths.


== Dogfood

This repo's add hook files each new task under `truth/tasks/{date:YYYY}-{date:MM}` as soon as it lands.

