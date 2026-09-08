#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "evaluate column conditions against the rendered rows",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 7,
  tags: ("cli",),
  estimate: (files: 7),
  assignee: "agent",
  status: proposed(2026, 9, 8),
  links: depends-on("add-boolean-literals-to-the-condition-language.typ"),
)

== Summary

A column's \`auto\` visibility means three different things, and all three are
buried in a Rust match (\`crates/mindtape-cli/src/render/columns.rs\`,
\`auto\_significant\`): \`file\` means "was the column named explicitly",
\`assignee\` means "do the rows disagree", everything else means "does any
cell have text". Making those expressible as condition predicates is what
lets one \`when\` expression replace the \`visibility\` / \`rules\` pair. This
task builds the domain and moves the evaluation to where the facts live; it
changes no config surface.


== Scope

The condition language is documented as ambient facts only
(\`crates/mindtape/src/condition.rs\`, module doc: "ambient environment facts,
never a task"). These predicates are not ambient — they depend on the
selected rows and on which columns the caller asked for. Extend the injected
environment rather than reaching for globals, keeping the module pure:

- \`nonempty()\` — any cell in this column carries text
- \`contested()\` — the rows disagree about this column's value
- \`named()\` — the column was explicitly requested (\`--columns\`, a view's
  \`columns\`, an \`\[\[ls.rules\]\]\` columns override)

\`ready\`'s current significance is \`readiness.is\_blocked\`, not literally
non-empty. Either add a fourth predicate or prove the two coincide (a
settled task renders blank, so they may) — do not assume it.

Resolution moves out of \`crates/mindtape-cli/src/ls\_columns.rs\` and into
\`resolve\_order\` (\`crates/mindtape-cli/src/render/columns.rs\`), which already
receives \`(config, handles, readiness)\` and so has the rows; it needs the
ambient environment threaded in as well and gains a \`Result\`. Both the table
and CSV renderers call it, which is what keeps them agreeing on the column
set.

\`Condition::parse\` dry-runs against \`ConditionEnv::probe\`, so the probe
grows fixed answers for the new predicates exactly as it already carries a
placeholder date. That is what keeps a typo like \`nonemtpy()\` a load-time
error naming the column rather than a column that silently never renders.

= Watch for

\`named()\` is a property of the REQUEST, never of the outcome — it reads
\`explicit\_order\`, which is set from what the caller asked for. Feeding it
from the resolved column set instead would make the predicate depend on the
decision it is an input to. Is there any path where a rule's \`columns\`
facet, itself resolved earlier, can reach \`named()\` after the fact?

