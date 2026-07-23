#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "preset save",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  status: done(2026, 7, 23),
  links: parent("init-presets.typ"),
)

== Summary

`mt preset save <name>` captures the current project's config and templates as a reusable user preset, and `mt preset ls` shows what exists.

This is what closes the loop for someone running many projects of their own. A personal defaults layer cannot help there — only the project's own config travels with the repository, so a shape left in a layer is invisible to anyone who clones it. Saving a preset turns a shape you arrived at once into something you can stamp into every later project as real, portable content.

== Scope

- `mt preset save <name>` writes `~/.config/mindtape/presets/<name>/config.toml` plus the project's `.mindtape/templates` beside it. One directory per preset, so a broken one breaks only itself — Vue CLI kept every preset in a single dotfile and one syntax error broke scaffolding entirely.
- No reverse-engineering of wizard answers from a config. The saved config is a raw overlay. Attempting to recover the answer set that would have produced it is guesswork, and a wrong guess silently changes what the preset means.
- `mt preset ls` lists built-in and user presets together, marking which user preset shadows which built-in, since shadowing is otherwise invisible until it surprises someone.
- Print the destination path on save. Vue CLI's save gave no feedback and users filed issues asking where it went, then more asking how to delete one.
- `--force` to overwrite an existing user preset; without it, refuse and say what exists.
- Tests: save then resolve by name round-trips config and templates; save refuses to clobber without `--force`; `ls` marks a shadowed built-in; saving from a project with no templates directory works.
