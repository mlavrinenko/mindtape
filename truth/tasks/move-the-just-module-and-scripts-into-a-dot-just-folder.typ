#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "move the just module and scripts into a dot-just folder",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 8),
  links: depends-on("gate-formatting-of-just-module-files.typ"),
  status: wip(2026, 8, 14)[wave 3, delegated],
)

== Summary

Create `.just/` and relocate `docs.just` into it, so the repo root stops
accumulating build-tool files and the extracted scripts have somewhere to live.

== The one mechanical trap, verified

A module recipe runs with its working directory set to the *module file's*
directory, not the repo root. Verified on just 1.46.0: a `mod docs
'.just/docs.just'` recipe that runs `pwd` prints `<repo>/.just`. Every body in
today's `docs.just` shells out with a repo-root-relative path (`bash
www/generate.sh`, `bash docs/generate-md.sh`), and `#[working-directory("www")]`
resolves against that same directory — it fails with a confusing
`could not find the shell: No such file or directory` when `.just/www` does not
exist. Today's file avoids all of this only by sitting at the repo root, which
its own header comment documents.

The fix is one line at the top of the moved file:

```
set working-directory := '..'
```

Verified: with it, a plain recipe runs at the repo root, and
`#[working-directory("www")]` composes on top to give `<repo>/www`. Both bodies
and the attribute keep working unchanged.

What does *not* work, also verified: `#[working-directory(source_directory() /
"www")]`. just 1.46 accepts const expressions there but not function calls —
`error: Expected ',', '=', or ')', but found '('`.

== Scope

- `mod docs '.just/docs.just'` in the root Justfile; the explicit path is
  required once the file leaves the root.
- `set working-directory := '..'` in the moved file, with a comment stating the
  cwd rule above — this is exactly the "execution knowledge" that belongs in the
  file rather than in prose elsewhere.
- Create `.just/scripts/` with a `.gitkeep` or the first extracted script, so
  the sibling extraction tasks have a landing place.
- `outdatty.yaml` names `docs.just` as a `dev-docs` source. Update the path.
  That group's own comment already argues the principle — a recipe body moving
  out of the Justfile must not escape the drift gate — so the same reasoning
  applies to `.just/scripts/*.sh` as they arrive.
- Any Justfile edit busts `dev-docs`, whose seven `docs/src/` dependents must be
  reviewed and re-confirmed with `just outdatty-update` in the same commit.
- Sweep for references to the old path: `.mmz/config.yaml`, the GitHub
  workflows, `flake.nix`, `contrib/`. Recipe *names* do not change here, so most
  references should be unaffected — confirm rather than assume.
- `www/generated/just.typ` and the gate table must come out byte-identical. A
  rendered doc diff means something in the mangling or the dump changed, and is
  a finding, not something to regenerate past.
