#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Show only the forbidden gates in mt explain",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a session sees the refusal before the work, not after.],
      enforced-practice: helps[the gate is visible at orientation, not only at flip time.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the CLI says which of its own commands will refuse.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 17),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Run `mt explain` in a clone whose `[[on.flip]]` gate is not allowed.
+ 🔴 The index lists the hook's name, but not that `mt flip` will refuse with `gate not allowed`.
+ Run `mt flip done <task>`.
+ 🔴 The refusal arrives after the work, and the index never warned.
+ Run `mt allow`.
+ 🔴 It lists every command, allowed or not, so the one that will refuse does not stand out.
+ Run a `[[tasks.waits]]` checker that is not allowed.
+ 🔴 It refuses with `unknown: not allowed`, and the index calls every refusal a gate.

== TO-BE

- 🟢 Bare `mt explain` prints a `refusals:` block only for the commands the config runs that the user has not allowed: each `[[on.flip]]` `run` and each `[[tasks.waits]]` checker.
- 🟢 An allowed command doesn't print, because allowed is the normal case.
- 🟢 A project with every command allowed doesn't print a `refusals:` block at all.
- 🟢 Each line says what to do: ask the user to run `mt allow`.
- 🟢 The JSON envelope carries the same list under `refusals`.

```text
refusals:
  on.flip: mmz --is-fresh --tag gate (not allowed; ask the user to run `mt allow`)
  waits: scripts/locked {id} (not allowed; ask the user to run `mt allow`)
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/index.rs`: the `refusals` block, its JSON, and the tests.
- `crates/mindtape/src/allow.rs`: expose the per-command verdict the block reads.
- `schema/output/explain/2.schema.json`: the `refusals` array.
- `www/content/cli/explain.typ`: the `refusals` block.
- `crates/mindtape-cli/tests/it/explain_e2e.rs`: a forbidden gate and an allowed one.

== Alternatives

- List every command with `(allowed)` or `(not allowed)`: allowed is the normal case, so the line is noise.
- Leave the refusals to `mt explain rules --for flip`: it lists the hook and its message, but not the command or the allow verdict.
- Refuse at flip time only: the work is already done, and the refusal costs the session.
- Call the block `gates:`: a `[[tasks.waits]]` checker refuses with `unknown: not allowed`, so `gates:` mislabels it.
- Print the refusal without a remedy: the skill says never run `mt allow`. The reader needs the next step.

== Out of scope

- `commits.run`. It is not gated by `mt allow`, and `mt explain rules --for commit` names its rules.
- The editors. `mt go` and `mt add --edit` are not a flip gate.
- A new allow file. The block reads the record `mt allow` already writes.
- The `[tasks.comments]` and `[rm]` policy refusals. The shape block owns them. They are not allow-record commands.

== Dogfood

This repo allows the gate. `mt explain` doesn't print a `refusals:` block. A fresh clone sees the gate before it flips.
