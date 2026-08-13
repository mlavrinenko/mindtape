#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "stop the generated docs writers racing under parallel check",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  estimate: (files: 2),
  status: done(2026, 8, 14),
)

== Summary

`just check` is `[parallel]`, and three of its arms write `www/generated/` concurrently. `www/generate.sh` opens with `rm -rf "$OUT"`, so it can delete the directory out from under `check-doc-facts` and `docs-md-check` while they read it. The gate then fails with "file not found (searched at www/generated/crate-map.json)" on work that is perfectly correct.

== Why it surfaced now

Every arm runs through `mmz`, so on a warm cache the three rarely run together. Any edit to the Justfile busts `gates-meta` and therefore *every* rule at once, which is exactly when all three run cold and concurrently. Observed three times in one session while landing the Justfile attribute work — twice by a delegated agent, once here — and a bare retry always passed. An intermittently red gate is worse than a slow one: it trains you to re-run instead of to read.

== Scope

- `www/generate.sh` must stop wiping a directory it does not own. It removes only the artifacts it writes, or stages into a temp dir and swaps its own files across.
- `www/generate-facts.sh` writes each JSON through a temp file and an atomic rename, so a concurrent reader sees the previous complete file or the next one, never a truncated or absent one. It currently redirects straight onto the destination, which also means a failing `jq` (the new gate-group cross-check calls `halt_error`) leaves a zero-byte `gates.json` behind.
- Both scripts run under a shared `flock` on `www/generated/` if the two changes above are not enough on their own — reach for the lock second, not first.
- Reproduce before fixing: run the three arms concurrently from a cold `mmz` cache and get the failure, then confirm the fix by re-running the same way.
