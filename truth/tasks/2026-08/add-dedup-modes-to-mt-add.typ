#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "add dedup modes to mt add",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 6),
  links: depends-on("split-config-rs-before-it-blocks-the-next-change.typ")
    + related(
      "give-mt-add-machine-readable-output.typ",
    )[the outcome a skip has to be observable through],
  status: done(
    2026,
    8,
    8,
  )[fail/skip/add-suffix landed. Verified live in a scratch project: skip exits 0 without writing, add-suffix yields -2 then -3],
)

== Summary

`mt add` hard-errors on a stem collision (`add.rs:142`, `bail!("task {stem:?} already exists")`) and has no other collision policy. Every scripted filer has to pre-check the destination path itself or string-match an anyhow chain. That includes the quality-signal sweep this repo is about to grow and anyone else automating `mt add`. The collision policy should be the caller’s to state.

== Scope

- Add `--dedup <MODE>` with `fail` (today’s behaviour, the default), `skip` (exit 0, leave the tree untouched, report the existing path) and `add-suffix`.
- `add-suffix` reuses `naming::disambiguate`, which already appends an incrementing `-2`, `-3` suffix and caps at `MAX_DISAMBIGUATE_ATTEMPTS`. That path is reachable only when a project sets `[tasks.naming] stem`. `--dedup add-suffix` makes it reachable unconditionally. Stems are filenames, so the suffix is `-2`, not a parenthesised title suffix.
- Add a config default under `[tasks.naming]` so a project can state its policy once, with the flag overriding it.
- `skip` must be distinguishable from a write by the caller. Pair it with the machine-readable output task.
- Cover each mode end-to-end, including `skip` against an existing file and `add-suffix` exhausting into an error.
