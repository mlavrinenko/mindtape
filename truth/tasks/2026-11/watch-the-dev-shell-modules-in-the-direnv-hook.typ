#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Watch the dev-shell modules in the direnv hook",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a nix/dev edit reloads the dev shell before a gate runs. The Vale styles then match their source],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "max",
  status: done(
    2026,
    11,
    15,
  )[landed: .envrc watches nix/dev, and the direnv allow is fresh],
)

== AS-IS

+ Edit a file under `nix/dev/` (add a rule to `vale-styles.nix`).
+ 🔴 The next `just check` fails on the old Vale style, because the shell still points `target/vale/styles` at the previous store path:

```
$ just check
==> just docs::prose failed (log: target/check/docs__prose.log)
E100:view 'Comments' not found
```


== Environment

- `main` at `705a1274`, direnv 2.37.1 on NixOS, before `direnv reload` repointed `target/vale/styles`.


== TO-BE

- 🟢 A change under `nix/dev` makes the next `direnv exec` reload the dev shell and repoint `target/vale/styles`.

```
$ direnv exec . readlink -f target/vale/styles
/nix/store/<new>-mindtape-vale-styles
```


== Changes

- `.envrc`: add `watch_file nix/dev nix/dev/*.nix` after `use flake`, with a comment saying `use flake` watches only `flake.nix` and `flake.lock`.


== Out of scope

- A `use flake` fix upstream in direnv. It should watch the flake source paths, not only `flake.nix` and `flake.lock`.
- Rebuilding the shell on every command, or a wrapper around `direnv exec`.


== Dogfood

This repo's agent commands run through `direnv exec`, and the Vale style link under `target/vale/styles` must match `nix/dev` before a task-prose gate reads it.

