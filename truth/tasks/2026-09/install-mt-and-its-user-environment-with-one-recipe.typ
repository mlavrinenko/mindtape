#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "install mt and its user environment with one recipe",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: (),
  estimate: (files: 2),
  assignee: "agent",
  status: done(2026, 9, 3),
)

== Summary

Putting this checkout on a machine took four steps. The mt binary, the `@local/mindtape` Typst package, the generated JSON Schema and the agent skill were four separate manual acts. A personal `dotfiles` note recorded them, so a git pull left an unknown subset stale, silently, because a stale binary still runs and still answers. Give the repo one idempotent entry point that installs all four, with every body in `.just/scripts/` rather than inline in the Justfile.

== Scope

- \`just install\` installs binary + Typst package + JSON Schema + agent skills, idempotently
- \`just install-bin\` refreshes only the binary
- every recipe body lives in \`.just/scripts/\`, none inline
- one place derives the Typst package version and path, rather than copying them between install-lib and uninstall-lib
- \`defaults.toml\` is never force-rewritten: it is a hand-edited config layer, unlike the generated schema
