#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Give the dictionaries their own command",
  kind: "feat",
  priority: ice(
    impact: 4,
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
      self-describing: helps[a reader sees what a dict costs to load and where its entries come from],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 21),
)

== AS-IS

+ Declare a dict that includes two others, as `[dicts.verbs]` does here.
+ Ask how many entries it holds and where they come from.
+ 🔴 The only listing is `mt explain dicts`, which retires, and it shows the count without the composition:

```
$ mt explain dicts
mindtape-dev-verbs  318 entries, case fold
mindtape-verbs      500 entries, case fold
product             4 entries, case exact
source              16 entries, case exact
verbs               818 entries, case fold
```

== TO-BE

- 🟢 `mt dicts` lists every dictionary with its entry count, its case and its source.
- 🟢 A dict built from `include` names the dicts it includes, and the count reads as their sum.
- 🟢 `mt dicts <name>` lists the entries, as `mt explain dicts <name>` does today.
- 🟢 `--json` carries the same list.

```
$ mt dicts
mindtape-dev-verbs  318  case fold   inline
mindtape-verbs      500  case fold   inline
product             4    case exact  inline
source              16   case exact  inline
verbs               818  case fold   include: mindtape-dev-verbs, mindtape-verbs
```

== Changes

- `crates/mindtape-cli/src/cmd/dicts.rs`: the new command, its listing and its entry mode, moved from `cmd/explain/dicts.rs`.
- `crates/mindtape-cli/src/cmd/mod.rs`: register `dicts`.
- `crates/mindtape/src/config/dicts.rs`: expose each dict's source, inline or the names it includes.
- `crates/mindtape-cli/tests/it/dicts_e2e.rs`: the listing, the entry mode and the JSON shape.
- `schema/output/dicts/2.schema.json`: the source column bumps the version, since `dicts/1` is the shape `mt explain dicts` prints today. `schema/output/dict/` stays, and its description names `mt dicts <name>`.
- `www/content/cli/dicts.typ`: the new page.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- Keep the count in `mt config show` and drop the listing: the config holds the source, not the compiled entries, so the count would be a lie there.
- Fold the source into the existing `mt explain dicts` line: that command retires, and the source column is the part that states the cost.

== Out of scope

- The dict merge rules. `include` keeps resolving as it does.
- A dict's values in the listing. `mt dicts <name>` prints them.

== Dogfood

This repo declares five dicts, and `verbs` includes two of them. `mt dicts` shows 818 entries and names the two it sums.
