#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "lock the script attribute against a shebang regression",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 3),
  links: related(
    "adopt-the-script-attribute-for-the-shebang-recipes.typ",
  )[the conversion this would keep from unwinding],
  status: proposed(2026, 8, 16),
)

== Summary

`set script-interpreter := ["bash", "-eo", "pipefail"]` gives every `[script]`
recipe strict mode by construction, and the five converted recipes proved it.
Nothing stops the next multi-line recipe from being written the old way — a
`#!/usr/bin/env bash` body still runs, silently under no strict mode, which is
exactly the shape `count-tests` was in when it printed a wrong test count and
exited 0. The setting removed the mistake from the five recipes that existed,
not from the Justfile.

== Why a lock, not a convention

This repo already answers "a convention nothing checks" with a static
self-test, three times: `positional-args-check` asserts the four passthrough
recipes carry `[positional-arguments]` and forward `"$@"`, `arg-pattern-check`
asserts every `[arg(...)]` pattern rejects known-bad input, `size-fixture-check`
asserts the size gate's walker enters `.just/`. Each reads the state
statically — the `--dump --dump-format json` dump or a replica tree — and runs
no recipe body. The same gate for `[script]` fits that shelf exactly.

== Scope

- The assertion: no recipe body's first line is a `#!` shebang. Read it off the
  same `just --dump --dump-format json` the other two static gates already
  parse, walking `.modules` recursively so a recipe in `.just/docs.just` is
  covered too — a module is where the next long body is most likely to land.
- `.shebang` in the dump is NOT the field to read: measured on just 1.46, a
  `[script]` recipe reports `shebang: true` exactly as a `#!` body does (argued
  in `www/gates.jq`, where the same field means "this body is a script"). The
  discriminator is the body's own first line, or the `script` key in
  `.attributes`.
- Every `[script]` recipe must also be reachable by the setting: a module
  declaring `[script]` recipes without its own `set script-interpreter` gets
  just's default `sh -eu`, since a module does not inherit the root's — measured
  on just 1.46 under
  truth/tasks/adopt-the-script-attribute-for-the-shebang-recipes.typ. Assert the
  pairing, so `.just/docs.just`'s currently userless setting cannot be deleted
  as dead config and take strict mode with it.
- Decide whether this is a `[group("gate")]` arm or an advisory. A gate arm
  means the `check` dependency list, a `www/utils/gate-notes.typ` entry and a
  CONTRIBUTING.md row — the price the other three static self-tests already
  pay, and the reason to pay it is the same: an unenforced convention is the
  thing that rots.
- The Justfile is a `dev-docs` outdatty source, and a new gate lands a
  `.just/scripts/*.sh` beside it: reconcile that group's dependents and
  re-record with `just outdatty-update dev-docs`.
