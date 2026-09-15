#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let a config extend rule packs",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 7,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  kind: "feat",
  assignee: "agent",
  status: done(2026, 9, 16),
  links: depends-on("give-every-body-rule-a-name.typ"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a repo that extends a pack is policed by every rule in it, with no copy to fall behind],
      cheap-adoption: helps[adopting shared practice becomes one `extends` line plus overrides],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Put a `[[tasks.rules]]` entry in a `.mindtape/defaults.toml` one directory above a project, the ancestor layer `www/content/config.typ` describes under "The four layers".
+ Run `mt config show` in the project: the inherited rule is there.
+ Add one `[[tasks.rules]]` entry to the project's own `.mindtape/config.toml` and run `mt config show` again.
+ 🔴 The inherited rule is gone. Arrays replace wholesale, so keeping it means copying every inherited rule into every repo.
+ 🔴 No key switches off one inherited rule, or changes its threshold, while keeping the rest.


== TO-BE

- 🟢 A config extends packs and adjusts single rules:

```toml
extends = ["mindtape-tasks", "../shared/team-pack.toml"]
disable = ["imperative-title"]

[[tasks.body.content.rules]]
name = "word-caps"
path = [{ match = "^(Goals|Changes)$" }]
assert = { max-words = 300 }
```

- 🟢 A bare name resolves to a pack embedded in the binary under the reserved `mindtape-` prefix. A path resolves to a TOML file relative to the project root, written in config format.
- 🟢 Pack rules append to the project's rules, a project rule with the same `name` replaces the pack's, and `mt config show --origin` names the pack each rule came from.
- 🟢 A `disable` entry naming no rule is a config error.


== Changes

- `crates/mindtape/src/config_packs.rs` (new): resolve `extends` entries to embedded or path packs, reading a path pack with the same parser as a config.
- `crates/mindtape/src/config.rs`: load packs below the project config, append rule arrays by `name`, apply `disable` and same-name overrides.
- `crates/mindtape/src/config_packs_tests.rs` (new): red tests first: the Problem steps, a disabled rule, an override, a missing pack path.
- `crates/mindtape/src/lib.rs`: register the module.
- `schema/config.schema.json`: regenerate.


== Alternatives

- Merge every array across the existing four layers: it changes what every current config means, and still offers no `disable`.
- Copy packs in at `mt init`, as presets do: a copy never receives the next fix.

== Out of scope

- Packs fetched over the network: a path covers a shared checkout, and a URL would put infrastructure between `git clone` and `mt`.
- Arrays other than rules, such as the status legend: they keep replacing wholesale.
- A real pack: `ship-the-task-practice-as-the-mindtape-tasks-pack.typ`.


== Dogfood

Built in: this repo extends the `mindtape-tasks` pack in `ship-the-task-practice-as-the-mindtape-tasks-pack.typ`.

