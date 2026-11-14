#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Accept YAML values files for mt add",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 5),
  status: done(
    2026,
    7,
    31,
  )[landed extension+sniff dispatch and the shared yaml\_values converter],
)

== Summary

`mt add -f/--values` parses every source strictly as JSON (`add_values::collect_values` -> `serde_json::from_str`). A YAML values file or YAML on stdin then fails with "parse JSON from -". The CLI already depends on `serde_yaml_ng` and owns a YAML mapping -> RawValue converter for the `--edit` scaffold (`add_interactive::parse_buffer` / `yaml_to_raw`). That leaves the `-f` path as the only place YAML fails. Accept YAML there too.

== Scope

- Dispatch per source in `collect_values`: `.yaml`/`.yml` -> YAML, `.json` -> JSON, stdin `-` and any other extension -> sniff. For sniffing, try `serde_json` when the first non-space byte is `{`/`[`, else YAML. On a failed JSON attempt, retry YAML and report the error of the format that got furthest.
- Do not collapse to one YAML-only parser. libyaml rejects tab indentation and errors on duplicate keys, and `serde_json` accepts both today, so an existing tab-indented JSON values file would regress. Keep `serde_json` on the JSON arm.
- Remove the YAML -> RawValue conversion from `add_interactive.rs` into a shared module used by both the editor buffer and the `-f` path. jscpd fails on a duplicated block >= 70 tokens.
- Keep the semantic split. The editor path drops empty-string entries (blank means "use the default"), while a `-f` file must keep "" as a literal value. Filter empties in the editor caller, not in the shared converter.
- Reject a non-mapping root and a multi-document YAML stream with the same wording shape as the JSON arm ("values from X must be a mapping").
- Docs/help: the `--values` help line in add.rs and the list-slot FillError in crates/mindtape/src/template.rs ("-f/--values with a JSON array"). Also www/content/templates.typ, www/utils/cli-notes.typ, AGENTS.md, and skills/mindtape/SKILL.md where it references JSON.
- Tests: unit coverage per dispatch arm in add\_values\_tests.rs (extension, sniffed stdin, malformed input error text, tab-indented JSON still parses) plus an e2e piping YAML into `mt add -f -`.
