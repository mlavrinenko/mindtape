#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Guard the mt ls footer docs with the file that holds it",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "print-the-task-count-first-in-the-mt-ls-footer.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  assignee: "agent",
  status: accepted(
    2026,
    11,
    24,
  )[the fix works, and closing it needs \`mt allow\` for the new after-the-write commands],
)

== AS-IS

+ Re-recording the `cli-ls-docs` group while moving the count to the front of the `mt ls` footer
+ 🔴 #raw(".config/outdatty.yaml"): the `cli-ls-docs` group's comment says it guards "the line under the table", but its sources omit `crates/mindtape-cli/src/cmd/ls/rules_applied.rs`, which builds that line

```
$ sed -n 313,321p .config/outdatty.yaml
  # `mt ls`: the facets, paging and the next-page command, the line under the
  # table, the sort spec, and that there is no `--all`. `ls/rules.rs` resolves
  # the `[[ls.rules]]` cascade the pages describe.
  - name: cli-ls-docs
    source:
      - crates/mindtape-cli/src/cmd/ls/mod.rs
      - crates/mindtape-cli/src/cmd/ls/rules.rs

$ grep -n 'pub fn footer' crates/mindtape-cli/src/cmd/ls/rules_applied.rs
112:pub fn footer(
```

== TO-BE

- 🟢 `cli-ls-docs` lists `crates/mindtape-cli/src/cmd/ls/rules_applied.rs` as a source, so an edit to the footer text stales the group.
- 🟢 `just outdatty check` names `www/content/cli/ls.typ` and `skills/mindtape/references/filters.md` for review after a footer-only edit.

```
$ just outdatty check
[ stale ]  cli-ls-docs
    source changed:    crates/mindtape-cli/src/cmd/ls/rules_applied.rs
    review dependent:  www/content/cli/ls.typ
    review dependent:  skills/mindtape/references/filters.md
    confirm with:      outdatty update --group cli-ls-docs
```

== Changes

- `.config/outdatty.yaml`: add `crates/mindtape-cli/src/cmd/ls/rules_applied.rs` to the `cli-ls-docs` sources and say so in its comment.
- `.config/outdatty.lock`: re-record the `cli-ls-docs` group.

== Out of scope

- The other groups' source lists. Each gap gets its own finding.
- `crates/mindtape-cli/src/cmd/ls/set.rs`, which `sets-docs` already lists.

== Dogfood

A later edit to the footer text in `rules_applied.rs` stales the two pages that quote it, and `just check` asks for the review before the commit lands.

== Goals

+ #Helps(maintainer, to: match-docs-to-code)[the group guards the footer text only when the file that builds it is a source].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).
