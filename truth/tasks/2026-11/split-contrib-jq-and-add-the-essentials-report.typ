#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Split contrib/jq and add the essentials report",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
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
      self-describing: helps[the project shape is one script over the config, so it cannot drift],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 21),
  links: depends-on("show-the-resolved-config-in-mt-config-show.typ"),
)

== AS-IS

+ Look at `contrib/jq/`, which holds three export scripts.
+ Ask where a view over `mt config show --json` belongs.
+ 🔴 There is no home for one, and nothing tests the scripts that are there:

```
$ ls contrib/jq/
fixtures  github.jq  taskwarrior.jq  todo.txt.jq
$ grep -rn "contrib/jq" Justfile .just/
$
```

== TO-BE

- 🟢 `contrib/jq/import/` holds three export scripts.
- 🟢 `contrib/jq/reports/` holds the views over `mt config show --json --resolved`. `essentials.jq` is first.
- 🟢 `essentials.jq` prints the shape: roots and exclusions, statuses, actors, the priority framework and bounds, difficulty, estimate, kinds, link kinds, comments, extensions and templates. It also prints the commit trailer and run, the hooks, the rm filter, the tags, the views, the dictionaries, the derived fields, and each rule with its moments.
- 🟢 A gate runs each report against a fixture, and fails on a top-level config key the report drops.
- 🟢 The script says it is temporary, and names the condition that ends that: `git log --since="1 month ago" -- contrib/jq/reports/` empty.

```
$ mt config show --json --resolved | jq -f contrib/jq/reports/essentials.jq
{"roots": [{"path": "truth/tasks", "tag": "mt"}]}
```

== Changes

- `contrib/jq/import/github.jq`, `contrib/jq/import/taskwarrior.jq`, `contrib/jq/import/todo.txt.jq`: moved from `contrib/jq/`, unchanged.
- `contrib/jq/reports/essentials.jq`: the new view over `mt config show --json --resolved`. A comment says the moment table is this repo's own until it settles, and names the condition: `git log --since="1 month ago" -- contrib/jq/reports/` empty at a release review.
- `contrib/jq/fixtures/`: a config fixture and the expected output for each report.
- `.just/scripts/check-jq-reports.sh`: runs each report against its fixture and diffs the output, then fails when the live config carries a top-level key the report neither prints nor lists in its `dropped` array.
- `Justfile`: the `jq-reports` gate, carrying `[group("gate")]` and joining `_check-fast`, or `meta::gate-drift` fails.
- `www/content/cli/config.typ`: the recipe, and where a project's own report goes.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- Keep the reports in Rust, as `mt explain` does: the renderer drifts from the config, which is the bug this replaces.
- Put the reports in `.mindtape/` so a project overrides them in place: the reports are not settled enough to publish in the config package.

== Out of scope

- Bundling a report into the binary or the config package. The reports stay in `contrib/` until they settle.
- The three export scripts' behaviour. They move unchanged.

== Dogfood

This repo's `essentials.jq` prints the config shape `mt explain` prints today, from `mt config show --json --resolved`, and the four keys the index dropped appear.
