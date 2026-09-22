#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Derive the gitleaks walk bound from git's own ignore rules",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("gating",),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the bound can no longer drift from what git ignores],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[deletes a hand copy and the gate that policed it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 23),
)

== Problem

+ Run agents in `.claude/worktrees/` (globally gitignored), then `just check`.
+ 🔴 `check-secrets` walks every worktree's `target/`: minutes of CPU, then hundreds of `generic-api-key` hits in build caches: `.gitleaks.toml`'s path allowlist is a hand copy of the repo's two `.gitignore` files and cannot see `core.excludesFile`.
`gitleaks dir` reads no ignore file (upstream gitleaks#1195, open since 2023), so the copy and `_gitleaks-bound-check.sh`, which polices it, exist only to restate git.


== TO-BE

- 🟢 `check-secrets` skips exactly what git ignores (both `.gitignore` files, `.git/info/exclude` and the global excludes file) with no path list to keep in sync.
- 🟢 A tracked file inside an ignored directory is still scanned.
- 🟢 The control fixture proves the derived bound fires before the tree is trusted.

The generated config, written to a temp file per run:

```toml
[extend]
path = "/abs/repo/.gitleaks.toml"

[[allowlists]]
description = "gitignored (git ls-files -oi --exclude-standard --directory)"
paths = ['''^target(/|$)''', '''^www/generated(/|$)''']
```


== Changes

- `.just/scripts/check-secrets.sh`: the control fixture becomes a git repo with a `.gitignore`, scanned through the same derived bound, and must stay silent on the ignored copy (red first); arm two derives the bound from `git ls-files -oi --exclude-standard --directory` into a temp config extending `.gitleaks.toml`; arm four goes.
- `.gitleaks.toml` — the hand-copied path allowlist and its argument go; the `outdatty.lock` allowlist stays.
- `.just/scripts/_gitleaks-bound-check.sh`, `_gitleaks-bound-control.sh` — deleted, with their `.mmz/config.yaml` scopes and `outdatty.yaml` entries.
- `.gitignore`, `www/.gitignore` — comments that point at the bound check.
- `www/utils/gate-notes.typ`, `docs/src/contributing/gates.typ` — describe the derived bound.


== Rollback

Revert the commit; the hand-copied bound and arm four come back whole.

== Out of scope

- Patching gitleaks to read `.gitignore` itself: declined.
- Scanning history: out of scope for this gate, argued in `.gitleaks.toml`.


== Dogfood

`just check` runs it on every commit here.
