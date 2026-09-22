#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add mt set to write a task field from the CLI",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli", "mt"),
  estimate: (files: 8),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a field change goes through the validator instead of around it],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one writer, already tested, replaces a hand edit nothing checks],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 22),
)

== Problem

+ Run `mt add "Upgrade the driver"`, then try to give the new task a
  priority without opening the file.
+ 🔴 There is no command. `mt` writes `status` (`flip`), `links` (`link`)
  and `reviewed` (`review`), and nothing else: `priority`, `difficulty`,
  `kind`, `assignee`, `estimate`, `tags`, `due`, `start` and `title` have
  no CLI writer at all.
+ `skills/mindtape/SKILL.md` opens by warning that a hand edit skips what
  `mt` stamps, slugs, injects, resolves and validates. For nine of the
  fourteen core fields that warning names the only available path.
+ `skills/mindtape/evals/evals.json` eval 4 now grades that hand edit as the
  correct recovery, so the skill's own test suite teaches the workaround.


== TO-BE

- 🟢 `mt set <task> <field>=<value>` writes one field and validates before
  the write, the way `mt add` and `mt flip` already do.
- 🟢 `tags+=<value>` appends to a list field, matching `mt add --set`'s own
  spelling rather than inventing a second one.
- 🟢 An unknown field is refused by name with the writable fields listed, the
  way an unknown `--set` key already is.
- 🟢 A value the config rejects fails before the file changes, so a refused
  write leaves nothing to repair.


== Changes

- `crates/mindtape-cli/src/set.rs` — new `Args`/`run` pair modelled on
  `review.rs`, reusing `task_edit::upsert_named_arg` and the `pre_write`
  seam every other writer goes through. The red test comes first.
- `crates/mindtape-cli/src/cli.rs` — wire the subcommand.
- `crates/mindtape-cli/src/command_class.rs` — declare `set` a write
  command; the exhaustiveness check in `cli_dump.rs` fails until it is.
- `crates/mindtape-cli/tests/read_no_write_e2e.rs` — no entry needed, but
  an end-to-end arm elsewhere in `tests/` must cover a refused write leaving
  the file byte-identical.
- `www/utils/cli-notes.typ` — the entry `check-doc-coverage` demands.
- `www/content/cli.typ` and `skills/mindtape/SKILL.md` — the new verb in the
  loop table; the hand-edit warning can finally name a command instead of a
  file.


== Alternatives

- A `--set` flag on `mt review`. Rejected: review stamps a date, and
  overloading it hides a writer inside an unrelated verb.
- Covering every field including `priority: ice(...)`, `links:` and
  `status:`. Rejected as the expensive cut; those three have their own
  commands or their own argument shapes, and the cheap subset is the
  scalars and lists whose Typst literal is unambiguous.


== Out of scope

- `priority:` in framework form, `links:` and `status:`. Each has a command
  or an argument shape of its own; a second spelling would drift from it.
- Renaming a task, which moves the file and rewrites inbound links; filed
  separately.
- Rewriting eval 4 to grade `mt set`. That eval exercises the refusal path
  on a template with one slot, which stays true.


== Dogfood

This backlog is the fixture. The next task needing a priority correction
uses `mt set` instead of an editor, and `mt check` proves the file still
validates.

