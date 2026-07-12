#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Import/export docs and schema reference",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("docs", "www"),
  links: parent("import-tasks-from-other-trackers.typ")
    + depends-on("mt-import-command.typ"),
  status: proposed(2026, 7, 13),
)

== Summary

Document the on-ramp and, crucially, the export answer. MindTape ships no foreign
export command by decision — export is plain files in git plus lossless
`mt ls --json`. Make that a feature, not a gap: a cookbook page shows import via
jq and export via reverse jq / `jq` one-liners, so the "fair to allow import
without export" concern is answered in prose, not code.

== Design / contract

- Canonical schema reference page: every field, authored vs computed, the
  priority object shape, `body`, the `SCHEMA_VERSION`. This is the contract jq
  authors code against.
- Import cookbook: the pipe form
  `task export | jq -f contrib/jq/taskwarrior.jq | mt import`, the GitHub form,
  and a `jq -R` recipe for todo.txt (the not-first-class source). `mt import`
  flags (`--into`, `--to`, `--dry-run`, mapping overrides).
- Export cookbook: `mt ls --json | jq ...` recipes — to todo.txt, to a CSV/
  spreadsheet shape, to a Taskwarrior-import skeleton — framed as "your data is
  already portable; here is how to reshape it". State plainly that there is no
  `mt export` and why (files + lossless JSON already are the export).
- Keep agent-facing Markdown under 200 lines (linecop); split pages if needed.
  Docs live under `www/` (tola SSG) — follow its content conventions and the
  `PKG_VERSION` import discipline for any `#import` examples.

== Files

- `www/content/` page(s) for the schema reference and the import/export cookbook,
  wired into the site nav. Possibly a short `README.md` pointer.

== Acceptance

- Pages build with the site (`tola` / the docs build recipe) and pass linecop.
- Every command shown is copy-pasteable and matches the shipped `mt import` flags
  and the jq files' actual paths.
- `just check` green.
- Commit `Refs: truth/tasks/import-export-docs.typ`.

== Scope

- Schema reference + import/export cookbook pages, nav wiring.
- Not in scope: any code; the jq files themselves (their task owns them).
