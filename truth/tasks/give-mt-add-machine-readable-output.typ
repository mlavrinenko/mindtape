#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "give mt add machine readable output",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 5),
  links: related(
    "add-dedup-modes-to-mt-add.typ",
  )[the outcomes this format has to report],
  status: proposed(2026, 8, 8),
)

== Summary

On success \`mt add\` prints \`wrote \<path>\` as prose (\`add.rs:148\`). A caller that needs the written path — any script that files a task and then wants to link, open or report it — has to regex stdout. \`mt ls\` already answers this with a \`--format\` enum carrying \`json\`, \`path\` and \`path0\`; \`add\` should reuse it rather than grow a second convention.

== Scope

- Add \`--format \<FMT>\` to \`mt add\`, reusing the existing \`output::Format\` value enum where the variants make sense: \`text\` (today’s prose, the default), \`path\`, \`path0\`, \`json\`.
- The JSON object must state what happened, not only where: the written path and whether the run wrote, skipped or suffixed — so a dedup mode is observable without parsing prose.
- Keep \`--print\` distinct: it previews the task source, this reports the outcome.
- Cover each format, including the skipped and suffixed outcomes once dedup modes exist.
