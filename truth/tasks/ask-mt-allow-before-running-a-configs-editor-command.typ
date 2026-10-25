#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ask mt allow before running a config's editor command",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[no command a clone's config names runs before the user allows it],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
)

== Problem

+ Reviewing the packs milestone, where fetched packs may carry any config key
+ 🔴 #raw("crates/mindtape/src/allow.rs"): `mt allow` covers `[[tasks.waits]]` and `[[on.flip]]` `run` lines only, while `mt go` and `mt add`'s editor fill spawn the program a config's `[go] editor` or `[add] editor` names, ahead of `$VISUAL`, without asking

```
crates/mindtape/src/allow.rs:37:    let checkers = config.tasks.waits.iter().map(|checker| checker.run.clone());
crates/mindtape/src/allow.rs:43:        .filter_map(|action| action.run.clone());
crates/mindtape-cli/src/cmd/go.rs:51:    editor_command(config.go.editor.as_deref())
crates/mindtape-cli/src/cmd/go.rs:68:    let status = Command::new(program).args(rest).status().map_err(|err| {
crates/mindtape-cli/src/cmd/add/interactive.rs:122:    let status = Command::new(program)
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/allow.rs").

== TO-BE

- 🟢 `allow::commands` lists `[go] editor` and `[add] editor` beside the `run` lines, so a clone or a pack that sets one asks `mt allow` first.
- 🟢 Until then, `mt go` and the editor fill fall back to `$VISUAL`, then `$EDITOR`, and warn once naming the held command.
- 🟢 An editor set in the user's own config layer needs no approval.


== Changes

- `crates/mindtape/src/allow_tests.rs`: the red tests first: a project `[go] editor` is held, a user-layer one runs.
- `crates/mindtape/src/allow.rs`: list the two editor commands from the project layer and its packs.
- `crates/mindtape-cli/src/cmd/go.rs`, `crates/mindtape-cli/src/cmd/add/interactive.rs`: skip a held editor and fall back.
- `www/content/config.typ`: say the editor keys wait for `mt allow`.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Rollback

- Revert the commit: the editor keys run unasked again.


== Out of scope

- Other keys that spawn nothing: only a key whose value `mt` executes needs approval.


== Dogfood

This repo sets no editor key, so `mt go` here keeps using `$VISUAL`.

