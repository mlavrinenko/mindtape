// docs/src/contributing/commit-messages.typ — source for
// docs/contributing/commit-messages.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// Deep dive behind CONTRIBUTING.md's Commit Messages section: the trailer
// rule, the `flip-with-work` convention and the two setups for it. The
// always-loaded file keeps the five-line rule and points here for the rest.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/commit-messages.md",
)) <mindtape-md>

#doc-title[Commit Messages]

#link("https://www.conventionalcommits.org/")[Conventional Commits]. Match
the language of prior commits. Footer with a task reference:

```
feat(filter): evaluate expressions over live tasks

Refs: <stem>
```

MindTape tracks its own backlog as per-file task artifacts under
`truth/tasks/` (dogfooding itself — see `.mindtape/config.toml`). Point
`Refs:` at the driving task's stem, not its path — a path breaks when a task
moves. `.githooks/commit-msg` checks it (`mt check --commit-msg`). The footer
is how code finds its tasks: before editing a file, run
`mt ls 'touches("<path>")'` to read the tasks that shaped it. Architectural
and tooling decisions are task artifacts too; the ADR process is retired.

Commits that file or edit task artifacts themselves (and the backlog config
in `.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs
nothing.

A status flip rides with the work it claims (`flip-with-work`,
mindtape-tasks): `mt wip`/`mt done` staged alone, with no other change, is
refused at commit time. Two setups, same rule:

- Worktree (#just.fork()): fork first, off a clean `main`, then flip inside the
  worktree. `mt wip <stem>` and the first line of the change land in one
  commit; `mt done <stem>` lands in the last, named by that commit's own
  `Refs:` trailer.
- Single checkout: flip `wip` and leave it uncommitted; do the work; commit
  the flip together with the first work commit. Close the same way: leave
  `mt done <stem>` uncommitted and fold it into the last commit of the
  change.

Filing a task stays a standalone `task:` commit either way — it names no
work, so it carries no `Refs:` footer and no flip rides in it. An
architectural or tooling decision with no code of its own tags the task
`decision` and closes in that same standalone commit, the one exemption
`flip-with-work` grants a task-only close.
