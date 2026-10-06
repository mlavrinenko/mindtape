#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build the flake without staging the whole worktree in nix-build",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate run never changes what the next commit contains],
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
  status: proposed(2026, 10, 6),
)

== Problem

+ Running \`just nix-build\` in the background while committing part of a worktree's changes
+ 🔴 #raw("Justfile"): the \`nix-build\` recipe runs \`git add -A\`, so every unstaged change lands in whatever commit comes next

```
nix-build:
    git add -A
    nix build .#default
```
A commit that staged eight named files went in with eleven: the two docs pages left unstaged for the next commit were in it, because `just nix-build` had run first.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 `just nix-build` builds the current tree, new files included, and leaves the index as it found it.


== Changes

- `Justfile`: replace `git add -A` with `git add -A --intent-to-add`. A git flake reads tracked files from the working tree, so only new files need to be known to git, and their content stays unstaged.


== Rollback

- Revert the `Justfile` recipe.


== Out of scope

- `just release`, which runs this recipe on a clean tree where staging changes nothing.


== Dogfood

Every `just nix-build` in this repo, which CONTRIBUTING.md asks for after a `flake.nix` change.

