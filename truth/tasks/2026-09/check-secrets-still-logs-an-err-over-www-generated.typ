#import "@local/mindtape:0.3.0": *

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
  status: done(
    2026,
    9,
    8,
  )[arm two holds `target/www-generated.lock` across the walk, with a `flock -n` probe asserting the hold. 8/20 before, 0/20 after.],
  links: related(
    "check-secrets-races-the-docs-arm-over-www-generated.typ",
  )[the same race. That task closed the descent and the mmz half. This one closes the stat of the directory itself.],
)

== Summary

The walk bound closed half of this race and cannot close the other half.
Measured with `strace` on gitleaks 8.30: the scan calls
lstat("www/generated") and only then writes
`DBG skipping directory: global allowlist path=www/generated`, one syscall
before the log line. The `^www/generated(/|$)` form buys what its
comment claims: the walk never descends. The directory entry itself
is still stat-ed on every run, because the code consults the allowlist after
it fetches the info, not before. `just check` runs in [parallel], and
www/generate.sh `rm -rf`s that directory on entry. The window between
readdir(www) and that `lstat` still produces
`ERR skipping directory: could not get info error="lstat www/generated:
no such file or directory"` on a clean run that exits 0.
An ERR on a green security gate trains the reader to skim it, which is
the property that makes a real leak look like the same flake. The sibling
task check-secrets-races-the-docs-arm-over-www-generated.typ closed the
mmz half and the descent. The stat of the directory is what remains.


== Scope

- hold `target/www-generated.lock` across the tree scan, the same fd-9
  flock and `MINDTAPE_GENERATED_LOCK` handshake www/generate.sh,
  docs/generate-md.sh and check-doc-facts.sh already share. Check-secrets
  is a reader of www/generated (it stats it) and is the last one not
  taking the lock
- release the lock as soon as the scan finishes rather than at script exit.
  Every other arm of a [parallel] `just check` waits while the scan holds it.
- assert at runtime that the scan is actually holding it, so deleting the
  acquisition fails the gate rather than quietly restoring the race
- correct the three places that state the bound prevents the ERR. They are
  .gitleaks.toml's walk-bound comment, docs/src/contributing/gates.typ and
  www/utils/gate-notes.typ. The bound stays, because it still keeps the
  walk out of `target/`, and arm four still holds it to www/.gitignore
- only www/generated needs this. Measurement showed `tola build` rewrites
  www/public and www/.tola in place without replacing either `inode`. It
  also renames the two favicon rasters over the old ones. Every other allowlisted path
  stays put mid-scan.

