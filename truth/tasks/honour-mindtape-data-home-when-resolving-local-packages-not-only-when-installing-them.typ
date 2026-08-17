#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "honour $MINDTAPE_DATA_HOME when resolving @local packages, not only when installing them",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 3,
  ),
  difficulty: 5,
  tags: ("mt", "substrate"),
  estimate: (files: 3),
  links: related(
    "resolve-the-typst-package-from-the-checkout-not-a-global-symlink.typ",
  )[the fix whose gate had to reach for `XDG_DATA_HOME` because of this],
  status: proposed(2026, 8, 17),
)

== Summary

`$MINDTAPE_DATA_HOME` moves where `mt install` WRITES the `@local/mindtape`
package (`crates/mindtape/src/app_dirs.rs` → `init.rs`), but not where the
loader READS it. Resolution happens inside `typst-world`'s `resolve_package`,
which hardcodes `dirs::data_local_dir()` and knows nothing about the variable
— so the one knob documented as working on every platform, and whose whole
reason for existing is that `dirs` reads no environment on Windows, isolates an
install but not a load. Point it at a tempdir and `mt install` fills that
tempdir while `mt ls` keeps reading your real package directory.

== Scope

- Decide the seam. Either `typst-world` grows a package-root override on
  `HarvestWorld` (a crates.io release), or `package_overrides` in
  `crates/mindtape/src/loader.rs` synthesises a `mindtape` entry under
  `data_home()` when the variable is set. The first is the honest fix — the
  variable's contract is about a directory, not about one package name.
- `app_dirs.rs` and `typst-world` both use `dirs::data_local_dir()`, so the
  platform paths agree today. Only the env override diverges, which is why
  nothing has ever failed over it.
- Found while making the example fixtures resolve their prelude from the
  checkout. `just check-examples` empties the global package directory so a
  fixture's `[[typst.local-packages]]` entry cannot silently rot back, and it
  has to do that with `XDG_DATA_HOME` — `$MINDTAPE_DATA_HOME`, the variable
  that should have done it everywhere, has no effect on a read. That is why
  the gate's isolation is Linux-only today, and closing this would let the
  recipe say what it means on every platform.
