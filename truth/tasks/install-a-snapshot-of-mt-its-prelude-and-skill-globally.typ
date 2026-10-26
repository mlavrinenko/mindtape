#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Install a snapshot of mt, its prelude and its skill globally",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 5),
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
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
)

== Problem

+ Run `just install`, then edit `lib/prelude.typ` in this checkout.
+ 🔴 Every other project on the machine now evaluates its tasks against the edited prelude with the binary from the last install, because `.just/scripts/install-lib.sh` links `~/.local/share/typst/packages/local/mindtape/<version>` to `<repo>/lib`. `.just/scripts/install.sh` links the skills the same way.
+ Run an older `mt install` anywhere.
+ 🔴 It writes its embedded prelude through that link into `<repo>/lib/prelude.typ`.


== TO-BE

- 🟢 `just install` writes the release binary, that binary's own prelude and a copy of each skill, so the rest of the machine runs one matched snapshot until the next install.
- 🟢 `mt` in a checkout of this repo still reads that checkout's `lib/`, as the loader already does for a `lib/` that is the `mindtape` package.


== Changes

- `.just/scripts/install-lib.sh`: run the installed `mt install --force` for the prelude in place of the link.
- `.just/scripts/uninstall-lib.sh`: remove the installed package dir, or a link an older install left.
- `.just/scripts/_typst-package.sh`: its header names its one remaining user.
- `.just/install.just`: `lib` installs the binary first, and the docs say snapshot.
- `.just/scripts/install.sh`: copy each skill in place of the link.


== Rollback

- Revert the commit and run `just install`, which links `lib/` and the skills again.


== Out of scope

- The project-level `.claude/skills/mindtape` link, which keeps the live skill in sessions in this checkout.
- What tinymist shows for this repo's tasks: it reads the installed snapshot, as for any other project.


== Dogfood

The other projects on this machine stay on the installed `mt`, prelude and skill, while sessions in this repo run the branch's.
