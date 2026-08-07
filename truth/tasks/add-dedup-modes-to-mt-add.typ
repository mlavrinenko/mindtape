#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "add dedup modes to mt add",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 6),
  links: depends-on("split-config-rs-before-it-blocks-the-next-change.typ")
    + related(
      "give-mt-add-machine-readable-output.typ",
    )[the outcome a skip has to be observable through],
  status: proposed(2026, 8, 8),
)

== Summary

\`mt add\` hard-errors on a stem collision (\`add.rs:142\`, \`bail!("task {stem:?} already exists")\`) and there is no way to ask for anything else. Every scripted filer — the quality-signal sweep this repo is about to grow, and anyone else automating \`mt add\` — has to either pre-check the destination path itself or string-match an anyhow chain. The collision policy should be the caller’s to state.

== Scope

- Add \`--dedup \<MODE>\` with \`fail\` (today’s behaviour, the default), \`skip\` (exit 0, write nothing, report the existing path) and \`add-suffix\`.
- \`add-suffix\` reuses \`naming::disambiguate\`, which already appends an incrementing \`-2\`, \`-3\` suffix and caps at \`MAX\_DISAMBIGUATE\_ATTEMPTS\`. Today that path is only reachable when \`\[tasks.naming\] stem\` is set; \`--dedup add-suffix\` makes it reachable unconditionally. Stems are filenames, so the suffix is \`-2\`, not a parenthesised title suffix.
- Add a config default under \`\[tasks.naming\]\` so a project can state its policy once, with the flag overriding it.
- \`skip\` must be distinguishable from a write by the caller — pair it with the machine-readable output task.
- Cover each mode end-to-end, including \`skip\` against an existing file and \`add-suffix\` exhausting into an error.
