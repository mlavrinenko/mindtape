#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Declare and ship the prelude versions each mt supports",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "substrate", "tooling", "stateless"),
  estimate: (src-files: 5),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[an upgraded mt keeps reading the preludes it still understands, so a bump need not rewrite a working import],
        enforced-practice: helps[an unsupported prelude is refused at load with one message naming the supported versions],
        plain-files: helps[the supported set ships in the binary, so an old pin needs no registry or download],
      ),
      ..("readable-tasks", "implementable-tasks", "typst-over-markdown"),
    ),
    process: verdicts(
      (
        sound-codebase: helps[every listed prelude has a fixture that proves the binary still evaluates it],
        docs-match-code: helps[the task-files and config pages say which versions are supported],
      ),
      ..(),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 4),
  links: depends-on(
    "resolve-unchanged-task-files-against-a-past-trees-package-pin.typ",
  ),
)

== AS-IS

+ In a project whose tasks import `@local/mindtape:0.2.0`, run `mt ls` with 0.4.0, once with that package installed and once without it.
+ 🔴 The first run loads the old prelude without saying whether the binary still supports it. The second fails as if 0.2.0 could never work, though 0.4.0 may still read it.
+ The binary names only `PRELUDE_VERSION`, so no release states a compatibility floor or drops an old version deliberately.


== TO-BE

- 🟢 `PRELUDE_SUPPORTED` lists every prelude version this binary evaluates, including the current one.
- 🟢 Each listed version ships in the binary and `mt install` writes it, so old pins resolve everywhere.
- 🟢 A listed version loads silently. An unsupported one fails once, naming the file, the version and the set, with a rewrite or upgrade hint.
- 🟢 A fixture per listed version proves it still evaluates, so dropping support is deliberate.
- 🟢 `mt check --commits` warns and skips a past tree whose prelude it no longer supports.

```
$ mt check
error: tasks/old.typ imports @local/mindtape:0.1.0, which this mt does not support
note: supported: 0.2.0, 0.3.0
hint: rewrite the import to @local/mindtape:0.3.0
```


== Changes

- `crates/mindtape/src/loader/embedded_tests.rs`: the red test first, each listed version loads with no package directory.
- `crates/mindtape/src/loader/embedded.rs`: public `PRELUDE_SUPPORTED` / `prelude_supports`, one immutable embedded snapshot and manifest per listed version.
- `crates/mindtape/src/loader/preludes/<version>/`: `0.2.0` from `v0.3.0`, `0.3.0` from `lib/`; each release archives its outgoing current here.
- `crates/mindtape/src/loader/pins.rs`: gate every spec before `StatePackages`, pins and disk.
- `crates/mindtape-cli/src/cmd/init/mod.rs`: `mt install` writes every listed version; a symlink stays unless `--force`.
- `crates/mindtape-cli/src/cmd/install/hint.rs`, `src/util.rs`, `cmd/check/templates.rs`: one diagnostic per unsupported version, first file and set.
- `crates/mindtape-cli/tests/it/{first_run_errors,install_e2e}.rs`, `src/cmd/eval_tests.rs`: update the `0.1.0` fixtures to the unsupported diagnostic; keep `99.99.99` as newer.
- `www/content/task-files.typ`, `www/content/config.typ`, `CHANGELOG.md`: the set and release step.

Contract: exact parsed `PackageVersion`s; `PRELUDE_VERSION` is current and highest. A listed version means eval reads its marker shape; layout is outside the gate. The gate applies to every world, including a past tree and a local pin. A current-tree import is a load error, so `mt check` fails and a listing warns and omits it. `mt check --commits` warns, skips that commit's checks, and does not report skipped tasks as missing refs. `0.1.0` is unsupported. Snapshots are immutable. One diagnostic per unsupported version names the first file and set.

== Alternatives

- A minimum version with a range: skipped versions and per-version tests make an explicit set more honest.
- Alias every supported version to the current prelude: no snapshots, but a pin would stop naming the code that runs.
- Keep only the current prelude and let `mt fix` rewrite: it rewrites pins that still work, which this design avoids.
- Rely on installed old packages: fresh clones and CI still fail.


== Out of scope

- Changing the DSL or migrating task bodies between preludes.
- Automatically deciding support. Each release adds or drops a version by hand with its fixture.
- Pruning unsupported packages already installed.


== Dogfood

This repo archives the outgoing prelude before a bump, lists it while a fixture passes, and drops it in the release that stops reading it.

