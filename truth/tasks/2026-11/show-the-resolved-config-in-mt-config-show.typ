#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Show the resolved config in mt config show",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 4,
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
      self-describing: helps[the config prints the values mt uses, so a default cannot hide],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Ask `mt config show` what level a rule with no `severity` reports at.
+ 🔴 The rule prints with no level, because the command prints the merged layers rather than the config `mt` uses:

```
$ mt config show | grep -B2 -A3 'name = "no-full-stop-in-titles"'
assert = '!matches(title, "\\.$")'
message = "drop the full stop at the end of the title"
name = "no-full-stop-in-titles"

[[tasks.rules]]
```

+ `[tasks.links]`, `[tasks.estimate]`, `[tasks.comments]`, `[add]` and `[format]` are absent the same way when no layer sets them, so `examples/minimal` shows none of the five.

== TO-BE

- 🟢 `mt config show --resolved` prints every key with the value `mt` uses, defaults included.
- 🟢 `--json` mirrors it. A report reads the resolved config rather than the layers.
- 🟢 `--origin` still reads the merged layers, because provenance is a different question.
- 🟢 The resolved output round-trips. Written as a config, it resolves to itself.

```
$ mt config show --resolved | grep -A4 'name = "no-full-stop-in-titles"'
assert = '!matches(title, "\\.$")'
message = "drop the full stop at the end of the title"
name = "no-full-stop-in-titles"
severity = "warn"
```

== Changes

- `crates/mindtape/src/config/mod.rs`: `Config` and the 67 structs under `config/`, `body/` and `rules/` that derive `Deserialize` alone gain `Serialize`. The output skips `extends`, `disable` and `pack-sources`, because they say how the layers combine, not what the config does. Every `Option` field gains `skip_serializing_if = "Option::is_none"`, because `toml` refuses a `None` value, and `config/active_rules.rs` already carries the pattern. `[params]` is absent from both views already, since `apply_params` rewrites each `param()` call before the merge.
- `crates/mindtape-cli/src/cmd/config.rs`: `ShowArgs` gains `--resolved`, conflicting with `--origin` and `--defaults`. `render_effective` branches on it, keeping the merged table for the default view and serializing the loaded `Config` for `--resolved`.
- `crates/mindtape-cli/src/cmd/config_tests.rs`: a rule with no `severity` prints its level, and the resolved output round-trips.
- `crates/mindtape-cli/tests/it/config_e2e.rs`: the flag, the JSON mirror and the round trip.
- `www/content/cli/config.typ`: the resolved view, and when to read it over the collected one.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- Make the resolved view the default. A reader who copies it into `.mindtape/config.toml` pins every default, so a later `mt` release cannot move one.
- A separate `mt config resolved` subcommand: one config, two commands, and the flag already says which view.

== Out of scope

- The collected view. It stays the default, because it is the one that reads as a config file.
- `--origin` and `--defaults`. Both keep their current source.

== Dogfood

This repo's `mt config show --resolved` prints `severity` on every rule and the seven `[tasks.links]` kinds, neither of which the collected view shows.
