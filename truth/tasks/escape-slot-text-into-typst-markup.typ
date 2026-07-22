#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "escape slot text into typst markup",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 7,
  ),
  tags: (),
  status: wip(2026, 7, 22),
)

== Summary

A text slot value written by `mt add` lands in Typst markup unescaped, so ordinary prose can produce a task file that fails to load the instant it is written. Found by dogfooding: a scope line containing the literal token create-star (an asterisk) opened strong emphasis and the task died with "eval error: unclosed delimiter". Same hazard for underscore, dollar, hash, square bracket, angle bracket and at-sign. The fix already exists one module over — `status_value.rs` has `escape_content` and applies it to status notes — so writes are inconsistent, not unprotected. This is a first-run quality bug in the exact place first runs happen: `mt add` reports "wrote <path>" and only then fails validation, leaving a broken artifact on disk.

== Scope

- Route every slot fill that lands in markup through the same escaping `escape_content` applies to status notes, and hoist that helper somewhere both call sites can share.
- Cover both fill paths: `--set` / `-f values.json` on the command line, and the YAML scaffold parsed back from `--edit`.
- Decide escaping per slot kind. Text and title slots are markup and must be escaped; int, float and choice slots are literals and must not be.
- On a write that fails to load, remove the written file or leave it and say so explicitly. Reporting "wrote" and then erroring, with a broken file left behind, is the current behaviour and is wrong either way.
- Tests: a title and a text slot each containing asterisk, underscore, dollar, hash, bracket, angle bracket and at-sign all round-trip to a loadable task whose field reads back literally; the same characters through `--edit`; a numeric slot is not escaped.
