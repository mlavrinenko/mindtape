#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add a just orient recipe for the session start",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[a session starts with one command that fits on a screen],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 21),
  links: depends-on(
    "split-contrib-jq-and-add-the-essentials-report.typ",
    "show-the-resolved-config-in-mt-config-show.typ",
    "give-the-templates-their-own-command.typ",
    "add-a-status-mode-to-mt-allow.typ",
  ),
)

== AS-IS

+ Start a session in this repo.
+ Ask what the project is, what is open, which templates exist and which commands the config allows.
+ 🔴 That is four commands, and the first one retires:

```
$ mt explain | wc -l
56
$ mt ls | wc -l
15
$ mt explain templates | wc -l
16
$ mt allow | wc -l
2
```

== TO-BE

- 🟢 `just orient` prints the project's shape, the default listing, the template list and the allow status, and nothing else.
- 🟢 It is cheap enough for every session: four `mt` calls under 200 lines, and only the listing reads the task corpus.
- 🟢 It reads `mt config show --json --resolved` through `contrib/jq/reports/essentials.jq`, and the shape follows the config. It pipes `mt ls --json` and `mt templates --json` through jq, so the listing and the template list each fit one line.

```
$ just orient
project:
  roots: truth/tasks (tag: mt)
  statuses:
    open: proposed, accepted, wip
    terminal: done, cancelled, failed
open: 12 of 1047 tasks
templates: decision, dev, ... (default: task)
allow: mmz --is-fresh --tag gate
```

== Changes

- `Justfile`: the `orient` recipe, under a `[group("session")]`, piping `mt ls --json` and `mt templates --json` through jq so each fits one line.
- `contrib/jq/reports/essentials.jq`: the shape the recipe prints.
- `www/content/quickstart.typ`: the session start, and the recipe as this repo's own.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- A shell script under `.just/scripts/`: the recipe is three commands, and `just` already groups them.
- A `mt orient` subcommand: it would need the allow join and the template scan, which the three commands already do.

== Out of scope

- Publishing the recipe to other projects. It is this repo's own, and a project writes its own.
- The rules. `mt config show` prints them, and the session start leaves them out.

== Dogfood

This repo runs `just orient` at the start of every session, and it prints the shape, the open task, the templates and the allowed command in one screen.
