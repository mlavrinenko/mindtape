#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve a task reference in the library, in one place",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 4,
  ),
  difficulty: 4,
  estimate: (src-files: 5),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[every command and tool accepts the same task references],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a new reference form or command touches one resolver],
      docs-match-code: helps[every <task> argument shares one help text],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
  links: related(
    "keep-a-printed-stem-hash-from-resolving-to-another-task-by-reserving-every-stem-and-filter-word.typ",
  )[the library resolver is where the reserved set lives],
)

== Problem

+ Reviewing the stem-hash commits 45d958bd to 9006d477
+ 🔴 #raw("crates/mindtape-cli/src/util.rs"): The library ships only \`StemHashes\`; resolution order, path rules and the hashed set live in the CLI, and several copies already disagree.

- `util.rs:162` reads `/` as a path; `eval.rs:62` also reads `\`.
- `rm_select.rs:68` repeats the exact-stem check; `link_plan.rs:141` targets take no hash.
- `ls_interactive/mod.rs:214` copies the normalisation and matches the printed hash, not the code.
- speconaut has its own resolver, `src/commands/task_resolve.rs:52`.
- `mt mv/set/review --help`: `slug, relative path, or absolute path`, no hash.
- `examples/large`: `mt eval task-00001` 1.74 s, `--json` 0.92 s; the project loads twice (`eval.rs:57`, `eval.rs:115`).


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/handle.rs").

== TO-BE

- 🟢 `mindtape` resolves a path, stem or stem hash to a task in one function that `mt` commands, `mt ls -i` and other tools call.
- 🟢 Every `<task>` argument comes from one shared `TaskRef` with one help text, and the concept has one name.
- 🟢 `mt eval` loads the project once.


== Changes

- `crates/mindtape/src/handle.rs`: `resolve_ref` and a public input matcher.
- `crates/mindtape-cli/src/util.rs`: `resolve_task` delegates to it.
- `crates/mindtape-cli/src/task_ref.rs`: the shared `TaskRef` argument, flattened into each command.
- `crates/mindtape-cli/src/eval.rs`: resolve through it and reuse the loaded project.
- `crates/mindtape-cli/src/ls_interactive/mod.rs`: match with the library matcher.


== Rollback

- Revert the commit; each command resolves on its own again.


== Out of scope

- Moving `rm_select.rs` and `mt link` targets onto it: mechanical fan-out, a follow-up once the API lands.
- Porting speconaut, which is speconaut's change.


== Dogfood

Agents here pass one reference form to every `mt` command.
