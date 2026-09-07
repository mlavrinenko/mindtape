#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "hold the root gitignore to the gitleaks walk bound",
  priority: ice(
    impact: 3,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 3,
  tags: ("gating",),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 9, 7),
)

== Summary

Arm four of \`.just/scripts/check-secrets.sh\` holds the \`.gitleaks.toml\` walk
bound to \`www/.gitignore\`, and stops there. The root \`.gitignore\` gets no such
check, and it has drifted: \`/.direnv\`, \`/result\`, \`/.mindtape/cache/\`,
\`/skills/\*-workspace\` and \`.mmz/cache/\` are gitignored scratch the bound never
names, measured by planting an \`aws-access-token\` under \`.direnv/\` and seeing
gitleaks report it. \`target/\` is the one that matters for the same reason
\`www/generated/\` did — \`just test\`, \`just clippy\` and \`just cover\` churn it in
parallel with the scan — and it is bounded, but nothing would notice if it
stopped being.


== Scope

- decide, entry by entry, which root-anchored \`.gitignore\` entries belong in
  the walk bound: \`/.cargo/\*\` carries a tracked exception
  (\`!/.cargo/mutants.toml\`) that bounding the directory would hide from the
  scan, \`/result\` is a nix symlink gitleaks skips anyway, and
  \`/skills/\*-workspace\` is a glob no path regex mirrors
- extend arm four to whatever survives that decision, or record the exemptions
  where the arm can read them, so the answer is a check rather than a habit
- keep every pattern in the \`(/|\$)\` form: \`^target/\` never matches the
  directory \`target\` itself, so the walk enters it and stat-s each child

