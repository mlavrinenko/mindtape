#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Install a snapshot of mt globally and the branch's lib in the dev shell",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 6),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[other projects run a matched `mt` and prelude from the version installed, whatever this checkout holds],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a worktree evaluates its own branch's `lib/`, so a prelude change is dogfooded where it is made],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
)

== Problem

+ Run `just install`, then edit `lib/prelude.typ` in this checkout.
+ 🔴 Every other project on the machine now evaluates its tasks against the edited prelude with the binary from the last install, because `.just/scripts/install-lib.sh` links `~/.local/share/typst/packages/local/mindtape/<version>` to `<repo>/lib`.
+ Run an older `mt install` anywhere.
+ 🔴 It writes its embedded prelude through that link into `<repo>/lib/prelude.typ`.
+ Run `mt check` in a `just fork` worktree.
+ 🔴 It reads the main checkout's `lib/`, not the branch's.


== TO-BE

- 🟢 `just install` writes the release binary and that binary's own prelude and skills as plain copies, so the rest of the machine runs one matched snapshot until the next install.
- 🟢 Inside the dev shell, `mt`, Typst and tinymist resolve `@local/mindtape` to the checkout's own `lib/`, in the main checkout and in every worktree.


== Changes

- `flake.nix`: the shellHook exports `TYPST_PACKAGE_PATH` as `<checkout>/target/typst-packages` and links its `local/mindtape/<version>` to the checkout's `lib/`.
- `.just/scripts/_typst-package.sh`: drop the dev shell's `TYPST_PACKAGE_PATH`, so an install targets the machine's package dir.
- `.just/scripts/install-lib.sh`: run the installed `mt install --force` for the prelude in place of the link.
- `.just/scripts/uninstall-lib.sh`: remove the installed package dir.
- `.just/install.just`: `lib` installs the binary first, and the docs say snapshot.
- `.just/scripts/install.sh`: copy each skill in place of the link.


== Rollback

- Revert the commit and run `just install`, which links `lib/` and the skills again.


== Out of scope

- The project-level `.claude/skills/mindtape` link, which keeps the live skill in sessions in this checkout.
- What `mt install` does with a symlinked package dir: `refuse-to-overwrite-a-symlinked-prelude-in-mt-install` decides it.


== Dogfood

Every session in this repo runs the branch's prelude, and the other projects on this machine stay on the installed one.
