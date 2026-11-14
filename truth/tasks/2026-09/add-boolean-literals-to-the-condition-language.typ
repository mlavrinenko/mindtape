#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "add boolean literals to the condition language",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 4),
  assignee: "agent",
  status: done(
    2026,
    9,
    9,
  )[code+tests+docs done. All gates pass except check-secrets, which fails pre-existing (confirmed via git stash on clean main). When the gate's own fixture runs `cd` outside the repo, `gitleaks` unloads from PATH, a direnv/sandbox artifact unrelated to this change. Forced past failing gate: "mmz --is-fresh --tag gate".],
)

== Summary

The condition mini-language has no way to write a constant. The parser rejects every bare identifier as a task field (\`crates/mindtape/src/condition.rs\`,
the \`VariableIdentifierRead\` arm), which is right for \`status\` and wrong for
\`true\`. Collapsing column visibility onto a single \`when\` expression needs
\`when = 'false'\` to turn a column off. The literals come first, on their own,
where they are a four-line change rather than one line buried in a cutover.


== Scope

In \`crates/mindtape/src/condition.rs\`, accept \`true\` and \`false\` as boolean
constants in boolean position. Put them ahead of the \`VariableIdentifierRead\` arm
that currently turns them into a \`TaskField\` error. Decide whether they are
also comparison values (\`env("X") == true\`). The recommendation is no:
\`env()\` yields a string, and a boolean on one side of a comparison is a
confusion to reject rather than to define.

\`evalexpr\` may already parse \`true\` as \`Operator::Const\`. In that case the
work is in \`eval\_bool\`'s \`Const\` arm rather than a new identifier case.
Check which before writing, since the two produce different tests.

Cover both literals in \`crates/mindtape/src/condition\_tests.rs\`. Check
that they compose (\`!false\`, \`true && !tty()\`). Check that a near-miss like
\`True\` still fails as an unknown identifier rather than silently reading as
false. Add them to the predicate table in \`www/content/cli.typ\` under
\`= Conditional rules\`.

= Watch for

\`Condition::parse\` dry-runs every expression against \`ConditionEnv::probe\`,
so a literal must evaluate there too. Does a constant-only expression like
\`'false'\` survive the probe without tripping the root-node arity check?

