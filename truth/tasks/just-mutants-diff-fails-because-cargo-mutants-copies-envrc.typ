#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "just mutants-diff fails on a clean tree because cargo-mutants copies .envrc",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "mutants"),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 9, 9),
)

== Summary

`just mutants-diff` cannot run here at all. It fails in the *unmutated
baseline*, so no mutant is ever tested and the recipe CONTRIBUTING.md asks
for before every PR silently stops being run — the failure looks like a
broken build rather than a broken environment.

cargo-mutants copies the whole tree, `.envrc` included, into
`/tmp/cargo-mutants-mindtape-*.tmp/`. direnv then meets an `.envrc` it has
never been told to allow, refuses it, and unloads the dev shell — so the
build in the copy runs without the nix toolchain. Most crates still compile
through the absolute `rustc`/`cargo` store paths, and the run dies at the
first `cc-rs` build script instead:

```
direnv: error /tmp/cargo-mutants-mindtape-G7Oogs.tmp/.envrc is blocked.
        Run `direnv allow` to approve its content
...
error occurred in cc-rs: failed to find tool "gcc": No such file or directory
*** result: Failure(101)
ERROR cargo build failed in an unmutated tree, so no mutants were tested
```


== Scope

Reproduce with `just mutants-diff` on any clean checkout with direnv hooked:
the baseline fails in ~35s, exit 4, and `gcc` is on PATH the whole time in
the shell that launched it.

The copied `.envrc` is the lever. `direnv allow` cannot be pre-run against a
path that does not exist yet, so the fix is to keep direnv out of the copy
or out of the build: `.cargo/mutants.toml` already carries the file set and
the `mutants` profile, and `.just/scripts/_mutants-env.sh` already exports
the speed knobs both mutants recipes share, so either is a place to put it.
`DIRENV_DISABLE=1` in `_mutants-env.sh` is the cheapest candidate and needs
measuring rather than assuming — it has to survive into the child cargo and
every build script, which is exactly what an unmeasured export does not.

Whatever lands must FAIL LOUDLY when it stops working. A baseline that
builds without the dev shell is the same silent-green shape as the gate
fixtures argued in CONTRIBUTING.md: the recipe reports a build error, and
the reader who has never seen it working reads that as their own broken
diff. `just check-full` does not cover this — mutation testing is
deliberately outside every gate — so the pin has to be part of the recipe.

