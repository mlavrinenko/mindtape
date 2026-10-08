#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse CLI output quoted by hand in doc prose and render it from captures",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 4),
  tags: ("found", "gating", "www"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a quoted error line in the docs is checked against a real run],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a changed diagnostic cannot leave its old text on a docs page],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 8),
)

== Problem

+ Reviewing the docs before release: templates.typ quotes a hint that mt add no longer prints
+ 🔴 #raw("www/content/templates.typ"): an inline span quoting mt output, such as an error: or hint: line, passes every doc gate, so the quoted text goes stale when the message changes

```
www/content/templates.typ:36  #raw("hint: template `dev` files kind=dev: mt add -t dev ...")
crates/mindtape-cli/src/cmd/add/hint.rs:91  hint: file it with the template that sets that field itself: mt add -t dev
```
`check-doc-spans` checks a span's `mt` command line through clap, and `check-doc-blocks` checks blocks; neither reads a quoted output line.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-spans.sh").

== TO-BE

- 🟢 A page quotes one line of `mt` output with a helper that takes a capture slug and a regex and renders the matching line of that capture; the build fails when no line matches.
- 🟢 `just check-doc-spans` fails on a literal inline span starting with `error:`, `warning:`, `hint:` or `note:` in the sources the doc-blocks manifest names, naming file and line and pointing at the helper. Its control page holds one such span and asserts the finding.

  ```typ
  // www/content/templates.typ
  #capture-line("add-template-hint", "^hint: ")
  ```

- 🟢 Every existing literal span of that shape in `www/content/` and `docs/src/` renders through the helper, `templates.typ:36` included.


== Changes

- `www/utils/ui.typ`: the capture-line helper, reading the record `www/capture.sh` writes.
- `.just/scripts/check-doc-spans.sh` (and its control): refuse the literal shape.
- `www/generate-*.sh`: add any capture the converted spans need.
- `www/content/templates.typ` and each other page with a literal diagnostic span: use the helper.
- `docs/src/contributing/documentation.typ`: name the rule beside "Use checked examples"; run `just docs md`.


== Rollback

- Revert the commit.


== Out of scope

- Prose that paraphrases behaviour ("suggests wip"); no gate can read that.


== Dogfood

This repo's own docs are the gate's input.

