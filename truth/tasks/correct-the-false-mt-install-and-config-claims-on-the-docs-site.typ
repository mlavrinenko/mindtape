#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Correct the false mt install and config claims on the docs site",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "www"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a new user upgrading or naming files is not misled],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[three install and config claims match the code],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Reviewing the docs before release against the 0.3.0 binary
+ 🔴 #raw("www/utils/cli-notes.typ"): three mt install and config statements are false

- `cli-notes.typ:163` vs `cmd/init/mod.rs:284`: the prelude is a copy under `local/mindtape/<ver>/`.
- `config.typ:67` vs `cmd/init/preset.rs:177`, `cmd/worktrees.rs:164`: read, never written.
- `config.typ:257`: `{date:%Y}` filed `fix-the-thing-%Y.typ`; tokens in `date/mod.rs:92`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/content/config.typ").

== TO-BE

- 🟢 The `mt install` notes say to re-run it after an upgrade so the prelude matches the binary, and which parts that refreshes.
- 🟢 `config.typ` says `presets/` and `worktrees.toml` are files you write in the config directory, which `mt` reads.
- 🟢 `config.typ` lists the `{date:FMT}` tokens, rendered from a fact the build derives from `date/mod.rs` if the facts machinery can carry it, else from a capture of a stem that uses each one.


== Changes

- `www/utils/cli-notes.typ`: the `install` note.
- `www/content/config.typ`: lines 67 and 257, rewritten with the surrounding paragraph.


== Rollback

- Revert the commit.


== Out of scope

- Changing what `mt install` writes.


== Dogfood

Not applicable: this repo uses the in-tree `lib/`, not the installed prelude.

