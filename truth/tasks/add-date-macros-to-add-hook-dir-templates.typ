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
    product: verdicts(
      (
        readable-tasks: helps[a dated folder says when a task was filed without opening git],
        cheap-adoption: helps[a dated layout needs no project helper],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the hook docs and the config token table share one grammar],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 13),
)

== AS-IS

+ Give an `[[on.add]]` hook a dated destination, `actions = [{ dir = "tasks/{date:YYYY}-{date:MM}" }]`.
+ 🔴 `mt add` refuses the whole config: `on.add[0].actions[0].dir: unknown dir macro: {date:YYYY}`.
+ A dated file stem already works under `[tasks.naming] stem`, but a dated folder needs a Typst `derive` function, and `typst-world` exposes no today to one.

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
- 🟢 An unknown macro, or a format that expands to a separator, is still refused before anything is written.

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

- `crates/mindtape/src/config/add_hooks_dir.rs`: accept the date macro family in a static `dir`, and validate `{date:FMT}` against the shared token grammar.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`: expand each date macro from `today_ymd()` and `format_iso`.
- `crates/mindtape-cli/tests/it/add_hooks_e2e.rs`: a pinned clock lands a task in the month folder, and a separator in a format is refused.
- `www/content/hooks.typ`: document the macros beside `{template}` and the field macros.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A project `derive` function: it cannot read today from `typst-world`, and every project repeats the same helper.
- A new `{filed}` task field: a model change where a folder is enough, and it cannot be backfilled for existing tasks.
- strftime tokens such as `%Y-%m`: they disagree with the naming grammar, where `%` already passes through literally.


== Out of scope

- Date macros in any other config value; only `dir` takes them.
- A `filed` task field or a backfill of filing dates.
- `mt import` and `mt recur` placement, which keep their own paths.


== Dogfood

This repo's add hook files each new task under `truth/tasks/{date:YYYY}-{date:MM}` as soon as it lands.

