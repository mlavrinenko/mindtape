#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 7, 31),
)

== Summary

mt add -f/--values parses every source strictly as JSON (add\_values::collect\_values -> serde\_json::from\_str), so a YAML values file or YAML on stdin fails with "parse JSON from -". The CLI already depends on serde\_yaml\_ng and already owns a YAML mapping -> RawValue converter for the --edit scaffold (add\_interactive::parse\_buffer / yaml\_to\_raw), so the -f path is the only place YAML is refused. Accept YAML there too.

== Scope

- Dispatch per source in collect\_values: \`.yaml\`/\`.yml\` -> YAML, \`.json\` -> JSON, stdin \`-\` and any other extension -> sniff (try serde\_json when the first non-space byte is \`{\`/\`\[\`, else YAML; on a failed JSON attempt retry YAML and report the error of the format that got furthest).
- Do not collapse to one YAML-only parser: libyaml rejects tab indentation and errors on duplicate keys, both of which serde\_json accepts today, so an existing tab-indented JSON values file would regress. Keep serde\_json on the JSON arm.
- Extract the YAML -> RawValue conversion out of add\_interactive.rs into a shared module used by both the editor buffer and the -f path (jscpd fails on a duplicated block >= 70 tokens).
- Keep the semantic split: the editor path drops empty-string entries (blank means "use the default"); a -f file must keep "" as a real value. Filter empties in the editor caller, not in the shared converter.
- Reject a non-mapping root and a multi-document YAML stream with the same wording shape as the JSON arm ("values from X must be a mapping").
- Docs/help: the --values help line in add.rs, the list-slot FillError in crates/mindtape/src/template.rs ("-f/--values with a JSON array"), www/content/templates.typ, www/utils/cli-notes.typ, AGENTS.md, and skills/mindtape/SKILL.md where it names JSON.
- Tests: unit coverage per dispatch arm in add\_values\_tests.rs (extension, sniffed stdin, malformed input error text, tab-indented JSON still parses) plus an e2e piping YAML into \`mt add -f -\`.
