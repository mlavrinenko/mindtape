#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse to overwrite a symlinked prelude in mt install",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a stale `mt` can no longer downgrade the checkout a developer links the package to],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
)

== Problem

+ Trying the new `mt init` in a temp dir, where `mt` resolved to an older `~/.cargo/bin/mt`.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/init/mod.rs"): `install_lib` writes `prelude.typ` whatever `--force` says, and `write_atomic` follows the package dir's symlink into the dev checkout, so an older binary silently rewrites `lib/prelude.typ` to the prelude it embeds.

```
$ readlink ~/.local/share/typst/packages/local/mindtape/0.2.0
<repo>/lib
$ git diff --stat lib/prelude.typ
 lib/prelude.typ | 53 +----------------------------------------------------
```
The rewritten file matched `lib/prelude.typ` at `3d7226bc`, byte for byte.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/init/mod.rs").

== TO-BE

- 🟢 When the package dir is a symlink, `mt install` leaves it alone and prints where it points, unless `--force` is given.
- 🟢 A plain package dir still gets the binary's prelude on every install, so an upgrade still lands.


== Changes

- `crates/mindtape-cli/src/cmd/init/mod.rs`: `install_lib` takes `force`, checks `symlink_metadata` on the package dir, and skips a link without it.


== Rollback

- Revert the `install_lib` change.


== Out of scope

- Which `mt` a shell resolves outside the dev shell: that is the user's PATH, not Mindtape's.


== Dogfood

This repo links its package dir to `lib/`, so `mt install` from any binary leaves the checkout untouched.
