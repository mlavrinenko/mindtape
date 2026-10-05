#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Validate doc mt spans with clap through a hidden mt subcommand",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 4,
  estimate: (src-files: 5),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a CLI change no longer fails an unrelated gate's control],
      docs-match-code: helps[a doc span is judged by the parser that runs it],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
  links: related(
    "read-the-doc-sources-through-one-typst-extractor-shared-by-every-check-doc-gate.typ",
  )[both rewrite check-doc-spans; land the extractor first so this reads spans from it],
)

== Problem

+ Reviewing commits e3a52225 and e551aca4
+ 🔴 #raw(".just/scripts/check-doc-spans.jq"): the gate re-implements clap's argv grammar in jq, skips a span whose \`mt\` follows a prefix, and its control expects the live CLI's flags

```
$ cat d.typ
A `cd x && mt lss` span, a `MINDTAPE_NOW=2026-01-01 mt ls --jsn` span, `$ mt lsq`, and `mt ls | mt lsp`.
$ spans d.typ | jq -f check-doc-spans.jq
check-doc-spans: d.typ:1: `mt ls | mt lsp`: no subcommand lsp; closest: ls
$ grep -n 'valid: -i' .just/scripts/check-doc-spans.sh
127:`mt ls -x`: mt ls has no flag -x; valid: -i, -n, -v, -w
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cli_dump.rs").

== TO-BE

- 🟢 A hidden subcommand judges one command line the way `mt` would parse it:
  ```sh
  $ mt _cli --check-argv 'mt @name ls --jsn'
  mt ls has no flag --jsn; closest: --json
  ```
- 🟢 It applies `desugar_argv`, then clap, then `shortcut_command` for a status verb, and accepts a missing required argument.
- 🟢 The gate finds `mt` after `cd x &&`, `VAR=1` and `$ `, and passes each command line to it.
- 🟢 The control runs against a frozen `cli.json`, so adding a short flag to `mt ls` cannot fail it.


== Changes

- `crates/mindtape-cli/src/cli_dump.rs`: add `--check-argv <LINE>`.
- `crates/mindtape-cli/tests/cli_dump_e2e.rs`: cover an unknown flag, the `@name` sugar, a status verb and a missing positional.
- `.just/scripts/check-doc-spans.jq`: drop `check_mt`; keep the config-key checks.
- `.just/scripts/check-doc-spans.sh`: extract command lines after a prefix, call `--check-argv`, and run the control on the fixture.
- `.just/scripts/check-doc-spans-control.json`, new: the frozen command tree the control reads.


== Rollback

- Revert the commit; the jq walker and the live-CLI control return.


== Out of scope

- Reading spans out of Typst, filed as the shared extractor task.
- Config-key checks, which already read the schema.


== Dogfood

This repo's `just check` judges every `mt` span in its docs with the binary's own parser.

