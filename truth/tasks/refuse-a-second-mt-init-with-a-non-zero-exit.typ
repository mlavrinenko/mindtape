#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a second mt init with a non-zero exit",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer re-running init sees a refusal, and a script sees exit 1],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 19),
)

== Problem

+ Re-running `mt init` in a project that already has a config
+ 🔴 #raw("crates/mindtape-cli/src/cmd/init/mod.rs"): `scaffold_config` prints its refusal with `println!` and returns `Ok(false)`, so `mt init` over an existing config exits 0 with no `error:` prefix and writes nothing

```
$ mt init --no-input; echo "exit=$?"
./.mindtape/config.toml already exists (use --force to overwrite)
exit=0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/init/mod.rs").

== TO-BE

- 🟢 `mt init` over an existing config exits 1 with `error: ./.mindtape/config.toml already exists` and `hint: pass --force to overwrite it`, built as a `Diag::refused`.


== Changes

- `crates/mindtape-cli/src/cmd/init/mod.rs`: return a `Diag::refused` from `scaffold_config` in place of the `println!` and `Ok(false)`.


== Rollback

- Revert the file.


== Out of scope

- `mt install`'s identical message in `cmd/install/mod.rs`, which overwrites by design and needs its own call.


== Dogfood

Not applicable, this repo runs `mt init` only in tests and examples.
