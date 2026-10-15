#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Lint shell scripts with shellcheck",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("gating", "tooling"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a shell syntax error or quoting bug fails locally, before git runs the script],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[every tracked shell script is linted, not just the workflow ones],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 15),
)

== Problem

+ Put a syntax error in `.githooks/pre-commit`, such as an unclosed `if`.
+ 🔴 Run `just check`. It passes, because no gate reads the shell scripts. Git finds the error when it runs the hook.
+ 🔴 `just check-workflows` runs shellcheck on workflow `run:` blocks only. The 70 other tracked scripts (`.githooks/`, `.just/scripts/`, `www/`, `contrib/`, `docs/`, `examples/`) get no lint at all.


== TO-BE

- 🟢 `just shellcheck` runs shellcheck over every tracked file with a shell shebang, and `just check` runs it in the fast stage.
- 🟢 A clean run prints nothing.
- 🟢 The scripts that exist today pass.

```yaml
scopes: {shell-scripts: ["**/*.sh", ".githooks/**", "examples/*/scripts/*"]}
rules: [{name: just shellcheck, tags: [gate], inputs: [shell-scripts, gates-meta]}]
```


== Changes

- `Justfile`: a `shellcheck` gate in `_check-fast`.
- `.mmz/config.yaml`: a `shell-scripts` scope and a `just shellcheck` rule tagged `gate`.
- `.shellcheckrc`: follow `source` through the script's own directory.
- `.just/scripts/*.sh`, `www/*.sh`, `contrib/*.sh`: fix each finding, with a reasoned `# shellcheck disable` where the code is right.
- `docs/src/contributing/gates.typ`: name the gate where it lists what the fast stage covers.


== Rollback

Revert the commit. The scripts keep their fixes.

== Out of scope

- Formatting shell with shfmt. Style is a separate gate.
- Linting the `run:` blocks in workflows. actionlint already does.


== Dogfood

Every `just check` here lints the shell scripts.
