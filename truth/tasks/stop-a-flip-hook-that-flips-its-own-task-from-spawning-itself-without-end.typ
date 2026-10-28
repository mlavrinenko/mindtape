#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop a flip hook that flips its own task from spawning itself without end",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
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
      sound-codebase: helps[a hook that flips its own task again is refused instead of forking without end],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 28),
)

== Problem

+ Fixing `stop-a-waits-checker-that-runs-mt-from-spawning-itself-without-end`, which named `[[on.flip]]` out of scope
+ 🔴 #raw("crates/mindtape-cli/src/cmd/flip/hooks_cmd.rs"): an `[[on.flip]]` `run` that runs `mt flip done <stem>` on the task being flipped reruns the same hook in the child, which spawns the next child, with no bound

```
[[on.flip]]
name = "loop"
when = 'into("done")'
actions = [{ run = ["<mt>", "flip", "done", "do-a-thing"], gate = true }]
```
`timeout 6 mt flip done do-a-thing` from `wip`, and 4 seconds in:
```
$ ps -eo pid,ppid,args | grep "debug/[m]t flip" | wc -l
188
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/flip/hooks_cmd.rs").

== TO-BE

- 🟢 A hook that flips the very task and status it runs for gets a refusal naming the loop, in one hop.
- 🟢 A hook that flips another task still runs that task's own hooks, so no gate is skipped.


== Changes

- `crates/mindtape-cli/tests/it/flip_chain_e2e.rs`: the red tests, each under a 20 s timeout: a gate flipping its own task is refused in one hop, two hooks closing each other are refused, and a gate closing another task runs that task's gate.
- `crates/mindtape-cli/src/cmd/flip/chain.rs`: the flips in progress, read from `MINDTAPE_FLIP_CHAIN`; refuse a repeated (task file, status) pair, and any flip 16 deep.
- `crates/mindtape-cli/src/cmd/flip/hooks_cmd.rs`: pass the chain plus this flip down to every `run` command.
- `crates/mindtape-cli/src/cmd/flip/mod.rs`: admit each target against the chain before any guard, `--force` or not.
- `www/content/hooks.typ`: say what a hook's own `mt flip` may do.


== Rollback

- Revert the commit.


== Out of scope

- `[[tasks.waits]]` checkers. `MINDTAPE_WAITS_CHECKER` already stops them at one level.
- Refusing every nested flip. A hook that closes a parent task is a fair use, and its gates must still run.


== Dogfood

Not applicable. This repo's `[[on.flip]]` hooks run no `mt flip`.
