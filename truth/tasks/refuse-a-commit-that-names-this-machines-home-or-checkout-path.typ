#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a commit that names this machine's home or checkout path",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 3),
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
      sound-codebase: helps[a leaked path is refused at the commit that would carry it, not found after a push],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[the pre-commit and commit-msg hooks refuse \$HOME and the checkout's paths],
)

== Problem

+ File a finding with `mt add --template finding` and paste a failing `cp` as its evidence, as `keep-the-docs-captures-from-copying-examples-api-service-while-another-gate-writes-its-cache` was filed.
+ Commit the task file.
+ 🔴 `git commit` takes it, though the evidence names the author's `$HOME`: three task files carried a home path until a history rewrite removed it. No gate reads `truth/tasks/`, and gitleaks drops a `/home/<user>/` rule, as `.just/scripts/crap-baseline-check.sh` measured.


== TO-BE

- 🟢 `git commit` refuses a staged file or a message that holds this machine's `$HOME` or the primary checkout's root, naming each file and line.
- 🟢 A path outside both, such as `~/.cargo/bin/mt` or a test's `/home/u/repo`, commits as before.


== Changes

- `.githooks/local-paths`: the needles, `$HOME` and the primary checkout's root, and a scan of the index or of one file.
- `.githooks/pre-commit`: scan the index.
- `.githooks/commit-msg`: scan the message.
- `docs/src/contributing/gates.typ` and `docs/src/contributing/commit-messages.typ`: say what each hook refuses.


== Rollback

- Revert the commit. No config key or file format changes.


== Out of scope

- Another machine's paths, such as CI's `/home/runner`. Only the machine that wrote a path knows it is local, and a `/home/<name>/` pattern would refuse every test fixture that spells one.
- The three commits already on GitHub's `main` that name a home path. Removing them needs a force push, which is the owner's call.


== Dogfood

Every commit in this repo is scanned before it lands.
