#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report the values mt add injects",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a defaulted field the filer never saw cannot silently route the task],
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
  status: done(
    2026,
    9,
    30,
  )[\`mt add\` now prints a stderr note per config-spliced core field the caller did not set, naming the value, its origin (\`\[tasks.assignee\]\` env \`MINDTAPE\_PRINCIPAL\`, \`\[tasks.fields.kind\]\` default, or a spliced slot) and the \`--set \<field>=\<value>\` override; std documented in SKILL.md],
)

== Problem

+ Run `mt add -t dev …` with `$MINDTAPE_PRINCIPAL=agent`.
+ 🔴 The filed task carries `assignee: "agent"`, although the `dev` template never mentions `assignee`; neither the output nor the file says the value came from `[tasks.assignee]`'s env default rather than from the filer.
+ An agent told "file this for max" has no cue that the assignee was defaulted, and must already know to pass `--set assignee=max`.


== TO-BE

- 🟢 `mt add` prints one note per core field it filled that the template did not: the field, the value, and its origin (`[tasks.assignee]` env `MINDTAPE_PRINCIPAL`, a config default, a spliced slot).
- 🟢 The note names the override (`--set assignee=max`), and an explicitly set field draws no note.
- 🟢 The task file itself stays comment-free: `[tasks.comments] policy = "forbid"` bans the marker that would otherwise live there.


== Changes

- `crates/mindtape/src/add_slots.rs`: `RequiredSlot` carries how its slot resolves (env name, default, or none), so the CLI can name the origin.
- `crates/mindtape-cli/src/add.rs`: after the fill, print one note per injected field whose value did not come from `--set`/`-f`, with the value and the override spelling.
- `crates/mindtape-cli/tests/add_config_scaffold_e2e.rs`: a red-first test pinning the env-filled, default-filled and explicitly-set cases.
- `docs/src/skill.typ`, regenerated `skills/mindtape/SKILL.md`: one line under Filing tasks, if the note lands.


== Alternatives

- A `//` comment beside the value in the written task: rejected, `[tasks.comments]` forbids comments in task files, so `mt check` would reject the very file this task files.
- An `x-add` extension slot carrying provenance: rejected, it stamps filer metadata into every artifact and every consumer must learn to ignore it.
- `--describe` alone: rejected, it describes the template, not this run's fills, and nobody rereads it per filing.


== Out of scope

- Which fields are required, and their defaults.
- The interactive `--edit` prompts: the note is output, not a new question.


== Dogfood

The next `mt add` for another actor starts with that actor's assignee, not `agent`.

