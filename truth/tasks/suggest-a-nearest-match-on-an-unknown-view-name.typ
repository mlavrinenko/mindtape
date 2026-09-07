#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "suggest a nearest match on an unknown view name",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 4),
  assignee: "agent",
  status: proposed(2026, 9, 7),
)

== Summary

An unknown view name errors with a bare \`unknown view: \<name>\` and nothing
else — no nearest match, and no list of the views that do exist. Every other
unknown-name error in the CLI at least enumerates the legal values
(\`unknown column ...expected one of\`, \`unknown status ...configured legend\`,
\`unknown --set/-f key(s) ...known slots\`), so views are the outlier. Add a
shared nearest-name helper to \`crates/mindtape\` and wire it into the view
lookup sites.


== Scope

- Add a \`suggest\` module to \`crates/mindtape\` exposing a nearest-match
  helper over a candidate name and a list of known names, plus a formatter
  rendering a \`did you mean: \<name>?\` line and a \`known views: a, b, c\`
  line. \`strsim\` is already in \`Cargo.lock\` transitively via clap, so
  promoting it to a direct dependency of \`crates/mindtape\` adds no new
  build; a hand-rolled Levenshtein is acceptable if preferred.
- Only suggest above a similarity floor — a name resembling nothing must
  fall back to the enumeration alone rather than naming an unrelated view.
- Wire the CLI view lookups: \`crates/mindtape-cli/src/ls\_rules.rs:127\`,
  \`:242\`, \`:305\`, \`:344\` and \`crates/mindtape-cli/src/views.rs:32\`.
- Wire the config-validation view lookups:
  \`crates/mindtape/src/config\_ls.rs:321\`, \`:343\`, \`:365\` and
  \`crates/mindtape/src/config.rs:213\`.
- Deterministic output: ties resolve by the candidate list's own order, so
  the message never varies between runs.
- Tests cover a near miss (suggestion offered), a far miss (enumeration
  only), and an empty view set (neither line, message unchanged).
- Out of scope: the column / status / slot / tag / worktree sites — those
  are the follow-up task.

