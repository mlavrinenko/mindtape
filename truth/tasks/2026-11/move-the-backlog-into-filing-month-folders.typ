#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Move the backlog into filing-month folders",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a month folder gives a reader a chronological entry point],
        enforced-practice: helps[the dir rule holds the layout on every write],
      ),
      ..(
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the repo map and the hook docs name the same layout],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 13),
  links: depends-on("add-date-macros-to-add-hook-dir-templates.typ"),
)

== AS-IS

+ List the backlog: `ls truth/tasks | wc -l`.
+ 🔴 It prints 991 files in one folder, so a reader sees every task ever filed in one run and has no month boundary.
+ `mt add` writes new tasks to the flat first root, and no rule holds a folder layout.

```
$ ls truth/tasks | wc -l
991
```


== TO-BE

- 🟢 Every non-swept task sits under `truth/tasks/YYYY-MM/`, the month git records it was filed.
- 🟢 An `[[on.add]]` hook writes a new task to `truth/tasks/{date:YYYY}-{date:MM}`, and a swept task to `truth/tasks/swept/`.
- 🟢 Two `[[tasks.rules]]` hold the layout: a non-swept `dir` matches `^truth/tasks/[0-9]{4}-[0-9]{2}$`, and a swept `dir` is `truth/tasks/swept`.
- 🟢 `[tasks.derive-tags] collect = []` keeps month names out of tags, while the root `mt` tag stays.
- 🟢 The migration rewrites the 602 relative `../lib.typ` imports and every cross-month link, so `mt check` stays green.

```toml
[[on.add]]
name = "file-by-month"
when = '!has_tag("swept")'
actions = [{ dir = "truth/tasks/{date:YYYY}-{date:MM}" }]

[[tasks.rules]]
name = "file-tasks-by-month"
when = '!has_tag("swept")'
assert = 'matches(dir, "^truth/tasks/[0-9]{4}-[0-9]{2}$")'
```


== Changes

- `.mindtape/config.toml`: add the month and swept add hooks, the two `dir` rules, and `[tasks.derive-tags] collect = []`.
- `contrib/ask-refactoring.sh`: drop the explicit `--dir truth/tasks/swept`, so the swept hook chooses the batch folder.
- `truth/tasks/**`: move every file to its git filing month, or to `swept/` when tagged `swept`, and rewrite imports and cross-month links.
- `docs/src/agents.typ`: say the backlog is one folder per filing month, with a separate sweep inbox.
- `crates/mindtape-cli/tests/it/rules_e2e.rs`: lock a `matches(dir, ...)` rule that accepts a month folder and refuses a flat task.


== Alternatives

- Per-kind folders: `kind` duplicates a field filters already expose and breaks 291 cross-kind links.
- Area folders: most tasks carry two to four tags and tags change, so placement is not deterministic.
- Stay flat: 991 files in one folder and no dogfood of the `dir` rule.
- A month prefix in the stem: it changes every task's identity, while a folder leaves the stem stable.


== Out of scope

- Per-kind or area folders; this is a chronological archive.
- A `filed` date field; git records the month and the rule only checks the shape.
- Splitting `swept/` by kind or month; a sweep batch is one removable folder.


== Dogfood

This repo's own backlog is the fixture: `mt add` files into the current month, `mt check` holds the layout, and the next sweep lands in `swept/`.

