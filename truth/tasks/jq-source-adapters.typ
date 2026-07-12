#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "jq source adapters (Taskwarrior, GitHub)",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("tooling",),
  links: parent("import-tasks-from-other-trackers.typ")
    + depends-on("canonical-task-json-schema.typ", "mt-import-command.typ"),
  status: wip(2026, 7, 12),
)

== Summary

The sources, as jq programs — no Rust. Author `contrib/jq/taskwarrior.jq` and
`contrib/jq/github.jq` that transform each tracker's native JSON export into the
canonical task-JSON schema. These are the top-2 on-ramps for MindTape's audience:
GitHub Issues (widest reach, `gh issue list --json ...`) and Taskwarrior (the
CLI-neighbour, `task export`). Structural only, config-blind; `mt import` does
the per-project policy map.

== Design / contract

- `contrib/jq/taskwarrior.jq`: map `task export` JSON. `description` -> `title`
  (+ into `body` as a `== Summary` block so imported tasks pass a Summary rule);
  `status` (pending/completed/deleted) -> a neutral canonical status; `priority`
  (H/M/L) -> canonical priority; `project`/`tags` -> `tags`; `due` -> `due`;
  `uuid`/`urgency`/annotations -> an `x-tw` extra or dropped. Document each
  mapping decision inline.
- `contrib/jq/github.jq`: map `gh issue list --json
  number,title,body,state,labels,assignees,createdAt,url` output. `title` ->
  `title` + `body` -> `body`; `state` open/closed -> canonical status; `labels`
  -> `tags`; `number`/`url` -> `x-gh` extra.
- Both target the versioned schema from `canonical-task-json-schema`; keep them
  config-blind (they cannot read `config.toml` and must not try).
- Add `jq` to the dev/test shell (flake `devShell` packages) so tests and
  `just check` can run it. Note the addition in the report.

== Files

- `contrib/jq/taskwarrior.jq`, `contrib/jq/github.jq`.
- Fixtures: `contrib/jq/fixtures/{taskwarrior,github}.{in,out}.json`.
- Test harness (a `just` recipe or a Rust integration test that shells jq).
- `flake.nix` devShell (add `jq`). Docs one-liner belongs to `import-export-docs`.

== Acceptance

- Golden unit test per source: `<in>.json | jq -f <src>.jq` equals `<out>.json`
  (the canonical fixture). No Rust logic under test — pure transform.
- End-to-end: `<in>.json | jq -f taskwarrior.jq | mt import --into <tmp>` files
  valid tasks that pass `mt check`. Same for GitHub.
- `just check` green on the merged tree.
- Commit `Refs: truth/tasks/jq-source-adapters.typ`.

== Scope

- The two jq files, fixtures, tests, and the `jq` dev-shell addition.
- Not in scope: a `--via` wrapper (explicitly rejected), todo.txt (docs recipe
  only), any Rust per-source code.
