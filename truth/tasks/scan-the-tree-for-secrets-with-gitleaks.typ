#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "scan the tree for secrets with gitleaks",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 4),
  links: related("relativize-the-crap-baseline-s-committed-file-paths.typ")[
    where the measurement below came from
  ],
  status: proposed(2026, 8, 17),
)

== Summary

No gate reads the tree for committed credentials. gitleaks 8.30.1, measured over this checkout with a path allowlist for `target/`, `mutants.out`, `www/public/`, `var/` and `build/`, scans 8.31 MB in 1.6s — cheap enough for a `just check` arm; without that allowlist it walks `target/` and takes 1m44s over 2.09 GB. It reports one finding today, a `generic-api-key` false positive on `outdatty.lock:151` (the task filename `refactor-auth.typ`), so adoption costs an allowlist entry rather than a triage backlog.

== Scope

- a `.gitleaks.toml` extending the default ruleset, carrying the path allowlist above and the `outdatty.lock` false positive allowlisted by rule and path
- a gate arm, plus the `.mmz` scope, the `script-` scope and the gate-notes entry every arm needs
- out of scope: history scanning (`gitleaks git`), which reports the absolute paths this repo has already committed and would need a `.gitleaksignore` per past mint

== What it cannot do

A machine-local-path rule is dead under `[extend] useDefault = true`, measured under `--log-level trace`: the default config's global allowlist matches `/home/<user>/` and drops any custom rule's finding on it (`skipping finding: global allowlist allowed-regex=true`), and an extending config can only ADD allowlists, never remove one. Dropping the default ruleset to make the custom rule fire would leave a secret scanner scanning for no secrets. That job stays with `crap-baseline-check`, and this repo's own `/home/u`, `/home/user` test fixtures would need allowlisting anyway.
