#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt import command",
  priority: ice(
    impact: 8,
    confidence: 0.75,
    ease: 4,
  ),
  tags: ("cli",),
  links: parent("import-tasks-from-other-trackers.typ")
    + depends-on(
      "task-to-typ-emitter.typ",
      "canonical-json-to-task-mapping.typ",
    ),
  status: done(2026, 7, 12),
)

== Summary

Wire the pieces into the `mt import` subcommand: read canonical task-JSON, map it
onto config (`canonical-json-to-task-mapping`), emit `.typ`
(`task-to-typ-emitter`), validate each file before it lands, write under a root,
and report. Import is "`mt add` without a template, from a full record" — reuse
`mt add`'s destination resolution (`--into` / `--to`), dedupe, and
validate-before-land discipline.

== Design / contract

- New `crates/mindtape-cli/src/import.rs`, registered in `cli.rs`. Keep files
  under the 500-line cap; split (`import.rs` + `import_tests.rs`, helpers as
  needed) up front.
- Input: canonical JSON from a file arg or stdin (`-`). Accept both the bare
  array (`mt ls --json` default) and the `{ "tasks": [...] }` / `{ "tasks": ...,
  "applied": ... }` envelope; ignore `applied`.
- Flags: `--into <DIR>` (default first root, mirror `mt add`), `--to <SUBDIR>`
  (nest, drives derived tags), `--dry-run` (report + preview, write nothing),
  `--status <NAME>` default for records missing one, and the mapping overrides
  from the mapping layer (`--map k=v`, `--drop-unmapped`, or equivalent). No
  `--from`/`--via` source selector — sources are jq, upstream of the pipe.
- Per record: map -> emit -> write to `<root>/<slug>.typ` (dedupe stems, e.g.
  `-2` suffix or skip-with-report; pick and document) -> validate with the same
  gate `mt add` uses. On any validation failure, fail that record and report;
  `--dry-run` never writes.
- Summary report to stderr: N filed, M skipped, unmapped/dropped fields per task.

== Files

- `crates/mindtape-cli/src/import.rs` (+ `import_tests.rs`), `cli.rs` wiring,
  `--help` text; CHANGELOG entry.

== Acceptance

- `just check` green.
- Round-trip integration test (the milestone acceptance): `mt ls --json` piped to
  `mt import --into <tmp-root>` reproduces the backlog field-for-field including
  `body` (modulo `file`/`derived_tags`/computed `score`). Prefer an in-process
  test over a shell pipe.
- Tests: stdin and file input; bare-array and enveloped input; `--dry-run` writes
  nothing; a record that fails validation is reported, not left half-written.
- Commit `Refs: truth/tasks/mt-import-command.typ`.

== Scope

- The `mt import` command, its flags, dedupe, validate-before-land, round-trip
  test.
- Not in scope: authoring jq adapters (separate task), docs cookbook.
