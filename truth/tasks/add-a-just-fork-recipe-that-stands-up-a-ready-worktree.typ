#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "add a just fork recipe that stands up a ready worktree",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 3),
  links: related(
    "resolve-the-typst-package-from-the-checkout-not-a-global-symlink.typ",
  )[the one thing the first real fork turned up that the recipe cannot fix],
  status: done(
    2026,
    8,
    17,
  )[just fork \<slug> \[base\]: sibling worktree, direnv-allowed and dev-shell loaded, base read back out of the created tree; refuses a dirty HEAD, warms nothing (measured)],
)

== Summary

Standing up a worktree for a delegated agent is four fiddly steps today —
`git worktree add <path> -b <slug> <base>`, `cd`, `direnv allow`, then assert
the base is the commit the orchestrator meant. Only the last one is load-
bearing and it is the one always skipped: a worktree branched from the wrong
base ships a diff that passes its own gate and silently drops whatever the
real base carried. `just fork <slug> [base]` should do all four and refuse
loudly on a base mismatch.

== What this is NOT for

Warming the build cache. That was the assumed motivation and the measurement
says otherwise: the dev shell inherits `kache` as `RUSTC_WRAPPER` from the
host session (`flake.nix`'s comment at the `devShells.default` `mkShell`; the
`RUSTC_WRAPPER = ""` further down belongs to the WINDOWS CROSS shell alone,
which blanks it because a stale wrapper breaks that build). `kache stats`
reports a 99.7% hit rate over a 50 GiB content-addressed store, so a fresh
worktree's `cargo build` is mostly cache hits already.

Copying `target/` is the wrong lever anyway. This checkout's is 17 GB, the
filesystem is ext4 (`cp --reflink` fails: `Operation not supported`), so an
`rsync`/`cp` seed is a real 17 GB write per worktree, and `cp -al` hardlinks
would let a worktree's build corrupt the source tree's cache. Measure before
building anything here: if a cold worktree gate is already acceptable, the
recipe is pure ergonomics and should stay small.

== Scope

- A `fork` recipe: create the worktree on a new branch off `[base]`
  (defaulting to `HEAD`), run `direnv allow` in it, and print the path.
- The base assertion, which is the actual point:
  `git merge-base --is-ancestor <base> HEAD` plus a check that HEAD descends
  from it, failing with the two hashes named. A silent wrong-base worktree is
  the failure this recipe exists to prevent.
- Decide what else a forked worktree needs to be immediately usable, by
  running one and watching what breaks — candidates are `.mmz/cache/`
  (cold means every gate arm runs once, which is CORRECT, not a bug to
  paper over) and anything else gitignored that a gate reads.

== Gate-coupled dependents

A new recipe lands in `www/generated/gates.json` only if it carries
`[group("gate")]`; this one should not. It still needs a `.mmz/config.yaml`
scope if it gets a script body (`script-scope-check` enforces one per
`.just/scripts/*.sh`), and `outdatty.yaml` couples the `Justfile` to its
dependents.

== Why now

Filed after an orchestration run passed on worktree parallelism for two
reasons that both turned out to be wrong — a WIP-limit breach that was
actually self-inflicted (both tasks flipped in one shared checkout rather
than one per worktree) and an assumed-cold build cache that `kache` had
already warmed. The friction that remains is real but small, which is
exactly why the recipe should be judged on the base assertion rather than
on speed.
