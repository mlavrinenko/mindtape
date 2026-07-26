#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Make `mt init` write a portable schema directive",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 3),
  status: done(
    2026,
    7,
    26,
  )[default is now SCHEMA\_URL; --schema local opts into the XDG path; docs and tests follow],
)

== Summary

`mindtape::SCHEMA_URL` documents itself as 'the value `mt init` `--schema url` writes for a portable, checked-in config', but no such shortcut exists: `--schema` takes its argument verbatim, so `--schema url` writes the literal string `url`, and `SCHEMA_URL` is never referenced from the CLI at all. The default is worse for a tracked file — an absolute `file://` path into the caller's XDG dir, which is meaningless to anyone who clones the repo. Adopting mindtape in linecop hit both.

== Scope

- Accept the literal `url` for --schema and expand it to `SCHEMA_URL`, matching what the doc comment already promises.
- Reconsider the default: a checked-in config wants the published URL, with the `file://` XDG path as the opt-in for offline or pre-install use.
- Fix or delete the `SCHEMA_URL` doc comment either way, so it stops describing an unimplemented feature.
