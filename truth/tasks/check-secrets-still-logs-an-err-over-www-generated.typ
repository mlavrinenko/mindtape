#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "check-secrets still logs an ERR over www generated",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("gating",),
  estimate: (files: 6),
  assignee: "agent",
  status: proposed(2026, 9, 8),
  links: related(
    "check-secrets-races-the-docs-arm-over-www-generated.typ",
  )[the same race; that task closed the descent and the mmz half, this one the stat of the directory itself],
)

== Summary

The walk bound closed half of this race and cannot close the other half.
Measured with strace on gitleaks 8.30: the scan calls
lstat("www/generated") and only then writes
\`DBG skipping directory: global allowlist path=www/generated\`, one syscall
before the log line. So the \`^www/generated(/|\$)\` form buys what its
comment claims — the walk never DESCENDS — but the directory entry itself
is still stat-ed on every run, because the allowlist is consulted after
the info is fetched, not before. \`just check\` is \[parallel\] and
www/generate.sh \`rm -rf\`s that directory on entry, so the window between
readdir(www) and that lstat still produces
\`ERR skipping directory: could not get info error="lstat www/generated:
no such file or directory"\` on a run that reports no leaks and exits 0.
An ERR on a green security gate teaches the reader to skim it, which is
the property that makes a real leak look like the same flake. The sibling
task check-secrets-races-the-docs-arm-over-www-generated.typ closed the
mmz half and the descent; this is the stat that is left.


== Scope

- hold \`target/www-generated.lock\` across the tree scan, the same fd-9
  flock and MINDTAPE\_GENERATED\_LOCK handshake www/generate.sh,
  docs/generate-md.sh and check-doc-facts.sh already share: check-secrets
  is a READER of www/generated (it stats it) and is the last one not
  taking the lock
- release the lock as soon as the scan is done rather than at script exit
  — every other arm of a \[parallel\] \`just check\` is blocked while it is
  held
- assert at runtime that the scan is actually holding it, so deleting the
  acquisition fails the gate rather than quietly restoring the race
- correct the three places that state the bound prevents the ERR:
  .gitleaks.toml's walk-bound comment, docs/src/contributing/gates.typ and
  www/utils/gate-notes.typ. The bound stays — it is still what keeps the
  walk out of \`target/\`, and arm four still holds it to www/.gitignore
- only www/generated needs this: measured, \`tola build\` rewrites
  www/public and www/.tola in place without replacing either inode, and
  the two favicon rasters are renamed over, so no other allowlisted path
  vanishes mid-scan

