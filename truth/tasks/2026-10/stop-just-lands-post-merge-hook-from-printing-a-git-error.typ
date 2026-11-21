#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

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
      sound-codebase: helps[a clean land prints only lines an agent can trust],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    13,
  )[not a repo hook. Needs a worktrunk fix, see Changes.],
)

== AS-IS

+ Landing the snippet-extraction branch with `just land`
+ 🔴 #raw(".config/wt.toml"): after `just land` removes the worktree, the run prints `fatal: not a git repository`, though the land succeeded

The landing agent reported that the "fatal: not a git repository" line came from the post-merge build hook after the worktree had disappeared.


This fell outside the task that surfaced it, so I filed it here rather than
leave it in a closing report no one rereads. It belongs in
#raw(".config/wt.toml").

== TO-BE

- 🟢 A successful `just land` ends without a `fatal:` line.


== Changes

- `.config/wt.toml`: none. Not reproducible from this repo. worktrunk 0.74 runs `post-merge` as a background `wt hook run-pipeline` in the target worktree when the merge removes the branch's worktree, and every hook log under `.git/wt/logs/` is empty. Two lands (one rebased) ended without a `fatal:` line. Neither a hook nor an `.envrc` step here runs git from the removed directory. A `fatal: not a git repository` then comes from worktrunk's own runner or from the caller's shell, whose working directory is the removed worktree. Upstream, `wt merge` needs to start the background pipeline from the target worktree, not the removed one, and to silence a runner that fails after the removal. This repo has no part to change.


== Rollback

- Revert the hook change.


== Out of scope

- worktrunk's own "Cannot change directory" hint, which needs `wt merge --no-cd` upstream.


== Dogfood

Every `just land` in this repo ends without a git error.
