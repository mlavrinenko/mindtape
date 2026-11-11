#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Route every explicit add value through one alias map",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 4,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[an explicit value under an aliased slot is either honoured or refused, never dropped],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[one alias map serves flags, --set and the positional title],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
)

== AS-IS

+ Reviewing the add pipeline, where hook ownership already expands an aliased slot but the other explicit sources do not.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/add/pipeline.rs"): Alias expansion runs only for hooks, so a direct flag or the positional title can silently drop an explicit value under an aliased slot.

```
# template: kind: slot("type")
$ mt add --print --name alias-flag "Alias flag" --kind feat --set type=bug
  kind: "feat",
```

The `bug` value is gone. On a template with `title: slot("headline")`, `mt add "Alias title"` fails with `missing a required slot: 'headline'`. `--set headline=...` fills the slot only when `--name` also gives the file a stem, because `output_stem` still reads the core `title` key.


== TO-BE

- 🟢 Every explicit source resolves to the core field before precedence is decided.
- 🟢 A flag plus a `--set` token for the same core field, including through an aliased slot, is refused.
- 🟢 The positional and piped title fill the template's title slot whatever its slot name is.

```
$ mt add --print --name alias-title "Alias title"
  title: "Alias title",
```


== Changes

- `crates/mindtape-cli/src/cmd/add/pipeline.rs`: build one alias map and use it in `refuse_flag_and_set`.
- `crates/mindtape-cli/src/cmd/add/field_values.rs`: route core-field operations through the same map.
- `crates/mindtape-cli/src/cmd/add/title.rs`: resolve the positional and stdin title to the template's title slot, and keep the core `title` value the stem reads.
- `crates/mindtape-cli/src/cmd/add/target.rs`: make `output_stem` accept the resolved title slot, so an aliased title still mints a stem.
- `crates/mindtape-cli/src/cmd/add/tests.rs`: table test over `writable::WRITABLE` with an aliased slot.


== Out of scope

- Composite expressions, whose embedded slots the existing docs already say a hook write loses.


== Dogfood

The skill's alias example and this repo's own templates use the same alias path.

