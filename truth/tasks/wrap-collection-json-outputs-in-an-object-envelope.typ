#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Wrap collection JSON outputs in an object envelope",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli", "stateless"),
  x-goals: (
    product: verdicts(
      (self-describing: helps[every JSON response names its schema]),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the docs and jq examples follow the one new top-level path],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 7),
  links: depends-on("generate-a-schema-for-every-json-output.typ"),
)

== AS-IS

+ Run `mt board --json | jq 'type'` and `mt explain views --json | jq 'keys'`.
+ 🔴 The first prints `array`, the second a map of view names, so neither can carry `$schema`. `mt at --json`, `mt @work ls --json`, `mt @work board --json`, `mt packs install --json`, `mt add --describe=json` and `mt config show --json` have the same shape.

```
$ mt board --json | jq 'type'
"array"
```

== TO-BE

- 🟢 Each prints an object whose first key is `$schema` and whose payload sits under a named key: `projects`, `entries`, `sources`, `views`, `dicts`, `slots`, `config`.
- 🟢 The old array or map moves under that key unchanged, so only the top-level path changes.
- 🟢 `CHANGELOG.md` names the break and the one-line `jq` fix.

```
$ mt board --json | jq '.projects[0].project'
mindtape
```

== Changes

- `crates/mindtape-cli/src/cmd/board.rs`: wrap the rows in `projects`.
- `crates/mindtape-cli/src/cmd/at.rs`: wrap the list in `entries`; a named target stays flat with `$schema`.
- `crates/mindtape-cli/src/cmd/ls/set.rs`: wrap the per-project listings in `projects`.
- `crates/mindtape-cli/src/cmd/packs/install.rs`: wrap the report in `sources`.
- `crates/mindtape-cli/src/cmd/explain/views.rs`, `crates/mindtape-cli/src/cmd/explain/dicts.rs`: wrap the maps in `views` and `dicts`.
- `crates/mindtape-cli/src/cmd/add/values.rs`: wrap the slot list in `slots`.
- `crates/mindtape-cli/src/cmd/config.rs`: wrap the effective config in `config`.
- `www/generate-*.sh`, `www/content/*.typ`, `skills/mindtape/references/*.md`: update the `jq` paths.
- `schema/output/{board,at,set-ls,set-board,packs,views,dicts,describe,config}/1.schema.json`: new goldens.

== Alternatives

- Keep arrays and maps bare: a top-level array has nowhere to put `$schema`, so the uniform rule would have an exception per command.
- Add a synthetic `$schema` element to each array: it changes every consumer that loops over the elements.

== Out of scope

- Object outputs: the previous task.
- Renaming any payload field now that it sits under a key.

== Dogfood

This repo's docs captures, skill examples and `www/generate-*.sh` jq programs use `.projects`, `.views` and `.dicts`.
