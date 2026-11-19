#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add an on.rm hook for removal actions",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("cli", "gating", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a removal can run the project's cleanup, not a manual step.],
      enforced-practice: helps[a removal gate refuses before the removal deletes the file.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the CLI says what happens on removal.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the hooks page covers removal like flip and add.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
  links: related(
    "show-only-the-forbidden-gates-in-mt-explain.typ",
  )[the refusals block lists the gate],
)

== AS-IS

+ Read `[rm]` in `.mindtape/config.toml`.
+ 🔴 It has `default-filter` and `strategy`, but no hook runs on removal.
+ Read `[[on.flip]]` and `[[on.add]]`.
+ 🔴 They cover a status change and a new task, not a removal, so a project cannot run a cleanup, show a message, or gate a removal.
+ Run `mt rm <task>`.
+ 🔴 The removal deletes the file before any project policy can react.

== TO-BE

- 🟢 `[[on.rm]]` runs actions when `mt rm` removes a task, with the same `when` and `actions` shape as `[[on.flip]]`.
- 🟢 An action can print a message, list tasks or run a command. A command marked as a gate refuses the removal before it deletes the file.
- 🟢 `[rm] default-filter` and `[rm] strategy` stay: they are the command's config, not a hook.
- 🟢 `mt explain rules --for rm` lists the hook, and the `refusals:` block names a forbidden `on.rm` gate.
- 🟢 The hook merges across packs and layers like `[[on.flip]]`, with the same name and `disable` rules.

```toml
[[on.rm]]
when = 'has_tag("chore")'
actions = [{ run = "just clean", gate = true }]
```

== Changes

- `crates/mindtape/src/config/hooks.rs`: the `RmHook` type and the `on.rm` field.
- `crates/mindtape/src/config/active_rules.rs`: the `on.rm` section and its gate entries.
- `crates/mindtape-cli/src/cmd/rm/mod.rs`: run the hook before the removal.
- `crates/mindtape-cli/src/cmd/explain/rules.rs`: the `--for rm` moment.
- `www/content/hooks.typ`: the `[[on.rm]]` section.
- `www/content/config.typ`: the `[rm]` and `[[on.rm]]` split.
- `www/content/cli/explain.typ`: the `refusals:` line for an `on.rm` gate.
- `crates/mindtape-cli/tests/it/explain_e2e.rs`: a forbidden `on.rm` gate.

== Alternatives

- Move `[rm] default-filter` to `[on.rm]`: the default filter is the command's default argument, not a hook. The move conflates the layers.
- Add an `on.rm` action to `[rm]`: a second hook shape, where `[[on.*]]` is the convention.
- No hook: a removal is rare, and the manual cleanup is fine.

== Out of scope

- The `[rm]` strategy. It stays.
- The `[retire]` config. It stays.
- The `mt rm` selection. It stays.
- The `[[on.flip]]` and `[[on.add]]` hooks. They stay.

== Dogfood

This repo's `mt rm` doesn't run a hook. The task adds one, and the `refusals:` block lists a forbidden `on.rm` gate.
