#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "preserve argument quoting with positional arguments",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 5),
  status: done(
    2026,
    8,
    14,
  )[five recipes on positional-arguments, proven with a PATH-shimmed cargo dumping argv; mutants preamble factored into \_mutants-env.sh; new positional-args-check gate reads the dump statically since --dry-run cannot see a word-splitting bug],
)

== Summary

The variadic passthrough recipes interpolate their arguments as a bare string,
which re-splits on whitespace and re-globs. `[positional-arguments]` gives the
body a real argument vector instead.

== The failure

`test *ARGS`, `build *args`, `mutants *ARGS`, `mutants-diff *ARGS` and
`ask-refactoring *ARGS` all expand their parameter into the command line. An
argument containing a space arrives as two arguments; one containing a glob
character is expanded against the working directory. `just mutants -f "path with
space"` cannot work today.

Verified on just 1.46.0: with `[positional-arguments]`, a recipe body reading
`printf 'arg: %s\n' "$@"` receives `two words` as a single argument.

== Scope

- Apply to the five passthrough recipes and rewrite each body to use `"$@"`.
- `mutants` and `mutants-diff` also export `CARGO_MUTANTS_JOBS` and prepend to
  `RUSTFLAGS`, duplicated between them. Deduplicating that is adjacent and
  welcome, but `RUSTFLAGS` must not become a Justfile-level `export`: it is part
  of the cargo fingerprint, so exporting it to every recipe would invalidate the
  build cache for the whole workspace.
- `test *ARGS` has a documented contract worth preserving — a filtered `just
  test <filter>` deliberately bypasses `mmz`, so the cache record reflects only
  the full-suite invocation `just check` runs. Confirm that still holds.
- If the extraction tasks have landed first, the bodies are one-line script
  invocations and the arguments forward as `"$@"` to the script. Sequence
  accordingly rather than fighting over the same lines.
