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
  status: cancelled(2026, 9, 9)[not this repo's bug; the diagnosis in the
    Summary was wrong and the Correction section says why],
)

== Summary

`just mutants-diff` cannot run on this host. It fails in the *unmutated
baseline*, so no mutant is ever tested and the recipe CONTRIBUTING.md asks
for before every PR silently stops being run — the failure looks like a
broken build rather than a broken environment:

```
direnv: error /tmp/cargo-mutants-mindtape-G7Oogs.tmp/.envrc is blocked.
        Run `direnv allow` to approve its content
...
error occurred in cc-rs: failed to find tool "gcc": No such file or directory
*** result: Failure(101)
ERROR cargo build failed in an unmutated tree, so no mutants were tested
```

The title and the original diagnosis blamed the copied `.envrc`. That was
wrong — see Correction.

== Scope

Nothing here. The cause is entirely in the host environment and no change to
this repo can fix it; a `DIRENV_DISABLE=1` in `.just/scripts/_mutants-env.sh`
would paper over one recipe in one project while the same tax is charged to
every `gcc` invocation on the machine.

= Correction

The copied `.envrc` is a symptom, not the lever. Two measurements refute the
original reading:

- A temp directory with *no* `.envrc` at all yields the identical stripped
  PATH (10 entries, no `gcc`, no `cargo`) as one holding a blocked `.envrc`.
  So `direnv allow`, or keeping `.envrc` out of the copy, would have changed
  nothing but the error text.
- `cargo`, `rustc`, `gcc` and `cc` are all nixpkgs *bash wrapper scripts*
  (`file $(command -v cargo)` says so). `BASH_ENV=/etc/bash_env` is therefore
  sourced by every one of them, which arms the DEBUG trap that loads direnv.
  cargo-mutants chdirs the child into its temp copy, so the `cargo` wrapper's
  own `$PWD` differs from the loaded project, and the trap unwinds PATH to the
  pre-direnv base *before* exec'ing the real cargo. Every build script
  inherits that base, and `gcc` lives in the dev shell, not the base.

Confirmed by disarming the trap for the run: `__DIRENV_ARMED=1 cargo mutants
--in-diff` builds the baseline clean and tests both mutants (2 caught).

The same mechanism charges every in-project `gcc` call as well, where the memo
is supposed to short-circuit: 342.6 ms ± 33.2 against 29.3 ms ± 4.6 with the
trap disarmed (hyperfine, 10 runs each) — 11.7x, because `set -T` fires the
trap once per command inside the 277-line wrapper.

Filed against the host configuration instead (a dotfiles repo, `apps/bash.nix`,
`environment.etc."bash_env"`). Reopen this only if a fix lands there and
`just mutants-diff` still fails.
