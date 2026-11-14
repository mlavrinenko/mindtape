#import "@local/mindtape:0.3.0": *

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
  status: cancelled(2026, 9, 9)[not this repo's bug. The diagnosis in the
    Summary was wrong and the Correction section says why],
)

== Summary

`just mutants-diff` cannot run on this host. It fails in the *baseline*,
so no mutant is ever tested. The recipe CONTRIBUTING.md asks
for before every PR then silently stops running, and the failure looks like a
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
wrong. See Correction.

== Scope

Nothing here. The cause is entirely in the host environment, and no change to
this repo can fix it. A `DIRENV_DISABLE=1` in `.just/scripts/_mutants-env.sh`
would paper over one recipe in one project while the same tax hits
every `gcc` invocation on the machine.

= Correction

The copied `.envrc` is a symptom, not the lever. Two measurements refute the
original reading:

- A temp directory with *no* `.envrc` at all yields the identical stripped
  PATH (10 entries, no `gcc`, no `cargo`) as one holding a blocked `.envrc`.
  So `direnv allow`, or keeping `.envrc` out of the copy, would have changed
  nothing but the error text.
- `cargo`, `rustc`, `gcc` and `cc` are all nixpkgs *bash wrapper scripts*
  (`file $(command -v cargo)` says so). Every one of them then sources
  `BASH_ENV=/etc/bash_env`, which arms the debug trap that loads direnv.
  cargo-mutants moves the child into its temp copy. The `cargo` wrapper's
  own `$PWD` then differs from the loaded project. The trap unwinds PATH to the
  pre-direnv base *before* it runs the real cargo. Every build script
  inherits that base, and `gcc` lives in the dev shell, not the base.

Confirmed by disarming the trap for the run: `__DIRENV_ARMED=1 cargo mutants
--in-diff` builds the baseline clean and tests both mutants (2 caught).

Every in-project `gcc` call pays the same charge, where the memo
should short-circuit. The measured cost is 342.6 ms ± 33.2 against 29.3 ms ± 4.6 with the
trap disarmed. Hyperfine ran 10 runs each, for 11.7x. `set -T` fires the
trap once per command inside the 277-line wrapper.

Filed against the host configuration instead (a `dotfiles` repo, `apps/bash.nix`,
`environment.etc."bash_env"`). Reopen this only if a fix lands there and
`just mutants-diff` still fails.
