#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Complete mt init --preset from the preset registry",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a fresh machine reaches the starter shapes at once.],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 22),
)

== AS-IS

+ Wiring the per-argument completion table in truth/tasks/emit-shell-completions-from-the-binary-that-answers-them.typ
+ 🔴 #raw("crates/mindtape-cli/src/completion.rs"): `mt init --preset` is the one argument in the `SOURCES` table whose values an existing function can enumerate and that still records `Source::None`. The driving task named six project-derived sources, and presets are an install-level registry rather than a project value.

```rust
("init", "preset", Source::None("an install-level starter shape, not a project value; `mt preset ls` enumerates them")),
```
`crate::preset::list_builtins()` and `list_user_presets(user_preset_dir())` already enumerate exactly what `mt preset ls` prints, so an existing value source covers it.

This fell outside the task that surfaced it, so I filed it here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/completion.rs").

== TO-BE

- 🟢 Tab after `mt init --preset ` offers the built-in and user presets `mt preset ls` prints, each helped by its one-line description.
- 🟢 A user preset directory the process cannot read yields the built-ins alone, never an error.

== Changes

- `crates/mindtape-cli/src/completion.rs`: a `Source::Presets` reading `preset::list_builtins` and `preset::list_user_presets`, replacing the `Source::None` entry for `("init", "preset")`.
- `crates/mindtape-cli/src/completion_tests.rs`: the tests offer the built-ins and accept a missing user directory.

== Rollback

Restore the `Source::None` entry for `("init", "preset")` and drop the `Presets` variant. The rest of the code ignores it.

== Out of scope

- `mt preset save <name>`, which creates a preset that does not exist yet. An existing value cannot predict it.
- Completing a preset from a project the user has not created yet. `mt init` runs before any project exists, so only the registry is in reach.

== Dogfood

This repo runs `mt init` only when scaffolding a fixture. The payoff belongs
to a new user. The suite `just check` runs still exercises the completion,
against the installed preset list.
