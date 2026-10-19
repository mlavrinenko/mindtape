#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add shellcheck source directives to the mutants scripts",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 2),
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
      sound-codebase: helps[an editor lints the mutants scripts clean with no rc file],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 20),
)

== Problem

+ Run `shellcheck -x --norc .just/scripts/mutants.sh .just/scripts/mutants-diff.sh` from the repo root, or from `/tmp` with absolute paths.
+ 🔴 It prints `SC1091 (info): Not following: ./_mutants-env.sh` for `mutants.sh` line 14 and `mutants-diff.sh` line 16. An editor's shellcheck searches upward for `.shellcheckrc` only, so it hits this since the rc moved to `.config/shellcheckrc`, whose `source-path=SCRIPTDIR` was the only thing that resolved the source.

== TO-BE

- 🟢 `shellcheck -x --norc` on both scripts prints no SC1091, from the repo root and from `/tmp`.
- 🟢 `just lint shellcheck` still passes.

== Changes

- `.just/scripts/mutants.sh`, `.just/scripts/mutants-diff.sh`: a `# shellcheck source=_mutants-env.sh` and `# shellcheck source-path=SCRIPTDIR` directive before the `.` line.

== Rollback

- Revert the commit. The two directives are comments; no behavior or config changes.

== Out of scope

- Moving the rc back to the root, and the other scripts, which already pass without an rc.

== Dogfood

An editor opened on this repo shows no SC1091 in the two scripts.
