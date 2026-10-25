#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the dev shell's hook install inside this repo when GIT_DIR names another",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[entering the dev shell never rewrites another repo's git config],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 25),
)

== Problem

+ Export `GIT_DIR` naming another repo's `.git`, as git does for every hook it runs, and enter this repo's dev shell from that process.
+ The `shellHook` in `flake.nix` runs `just install-hooks > /dev/null 2>&1`.
+ 🔴 Every `git config` in the `install-hooks` recipe writes into the repo `GIT_DIR` names: its `core.hooksPath` becomes `.githooks` and it gains the `outdatty-lock`, `jscpd-baseline` and `crap-baseline` merge drivers. Its own hooks stop running, and the output is silenced.
This happened to speconaut on 2026-10-23. `git -C <dir> config` does not help: `GIT_DIR` wins over `-C`, checked with two scratch repos.


== TO-BE

- 🟢 `just install-hooks` writes only this repo's config, whatever `GIT_DIR`, `GIT_WORK_TREE` or `GIT_INDEX_FILE` the caller exports.


== Changes

- `Justfile`: run the recipe's git calls with `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` unset and the working directory pinned to `{{justfile_directory()}}`, for example through `env -u GIT_DIR -u GIT_WORK_TREE -u GIT_INDEX_FILE git -C {{justfile_directory()}} config ...`.


== Rollback

- Revert the commit. No config key or file format changes.


== Out of scope

- Other recipes that run git; this one alone runs on every shell entry and writes config.


== Dogfood

This repo's dev shell runs the recipe on every entry, so every contributor gets the fix.

