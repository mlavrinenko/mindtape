#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Fail the gates on a stale Vale style link",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("stateless", "tooling", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a stale style cannot pass a gate that claims to enforce the current rules],
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
  assignee: "agent",
  status: done(2026, 11, 16)[gate green, 42 of 42],
)

== AS-IS

+ Edit a Vale input, such as `.config/vale/local/Prose/words.txt`, without reloading the dev shell.
+ 🔴 `target/vale/styles` keeps the old build, and `lint::vale-levels` passes over the difference.

```
$ echo zzz-stale >> .config/vale/local/Prose/words.txt
$ just lint vale-levels
$ echo $?
0
```

== TO-BE

- 🟢 `.envrc` watches the Vale overlay and the verb dictionaries. An edit there makes direnv re-evaluate the shell at the next prompt and repoint `target/vale/styles` at the new build.
- 🟢 `lint::vale-levels` fails when the linked style differs from `.config/vale/local`, or when the built `Imperative.tengo` disagrees with the dictionaries, and names `direnv reload` as the remedy.

```
$ echo zzz-stale >> .config/vale/local/Prose/words.txt
$ just lint vale-levels
check-vale-levels: .config/vale/local/Prose/words.txt differs from the built copy; run direnv reload
error: Recipe `vale-levels` failed on line 68 with exit code 1
```

== Changes

- `.envrc`: watch `.config/vale/local` and `crates/mindtape/src/dicts` beside `nix/dev`.
- `.just/scripts/check-vale-levels.sh`: the drift check and its control fixtures.
- `.just/lint.just`: the recipe's doc line.
- `docs/src/contributing/documentation.typ`: the gate line in the documentation deep dive.

== Out of scope

- The cached nix evaluation that can still serve an old build. The gate reports the stale link, and forcing a fresh evaluation belongs in nix-direnv.

== Dogfood

This repo edits its vocabulary and rule overlay often. `just lint vale-levels` refuses a stale link, and `.envrc` reloads the shell at the next prompt.
