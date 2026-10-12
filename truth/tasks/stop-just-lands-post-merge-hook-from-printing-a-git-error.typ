#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop just land's post-merge hook from printing a git error",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.6,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[a clean land prints no error line an agent has to judge],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    13,
  )[not a repo hook; needs a worktrunk fix, see Changes],
)

== Problem

+ Landing the snippet-extraction branch with `just land`
+ 🔴 #raw(".config/wt.toml"): after `just land` removes the worktree, the run prints `fatal: not a git repository`, though the land succeeded

The landing agent reported: the "fatal: not a git repository" line came from the post-merge build hook after the worktree was gone.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".config/wt.toml").

== TO-BE

- 🟢 A successful `just land` prints no `fatal:` line.


== Changes

- `.config/wt.toml`: none. Not reproducible from this repo. Worktrunk 0.74 runs `post-merge` as a background `wt hook run-pipeline` in the target worktree when the merge removes the branch's worktree, and every hook log under `.git/wt/logs/` is empty. Two lands (one rebased) printed no `fatal:` line. No hook or `.envrc` step here runs git from the removed directory, so a `fatal: not a git repository` comes from worktrunk's own runner or from the caller's shell, whose working directory is the removed worktree. Upstream, `wt merge` needs to start the background pipeline from the target worktree, not the removed one, and to print nothing from a runner that fails after the removal. This repo has no part to change.


== Rollback

- Revert the hook change.


== Out of scope

- worktrunk's own "Cannot change directory" hint, which needs `wt merge --no-cd` upstream.


== Dogfood

Every `just land` in this repo ends without a git error.
