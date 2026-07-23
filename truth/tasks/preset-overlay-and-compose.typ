#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "preset overlay and compose",
  priority: ice(
    impact: 7,
    confidence: 0.75,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("cli", "substrate"),
  status: done(2026, 7, 23),
  links: parent("init-presets.typ"),
)

== Summary

Two pieces that make preset composition safe: an escape hatch for config the axes cannot express, and a hard error when two fragments fight over the same array.

The escape hatch matters because axes will never cover everything — custom link kinds, named views, body-header rules. Without somewhere to put them, a user hits a wall and abandons the preset system for a hand-written config, which is the outcome the whole feature exists to avoid.

The conflict error matters because MindTape's merge replaces arrays wholesale. Status legends, priority bands, difficulty anchors, views and roots are all arrays, so two fragments each declaring one cannot be merged — one silently wins. Nix's module system raises an evaluation error on equal-priority definitions rather than picking, and that is the behaviour that makes layered config debuggable. Helm went the other way on lists and the issue is still open years later.

== Scope

- A preset file may carry `[overlay]`: a raw config fragment merged last, after every axis fragment. Whatever the axes cannot say goes here.
- Composition happens at init time and flattens to one readable file. The result is the user's to edit — that is the advantage of a scaffold over a live link, and it is why no runtime preset-stacking mechanism is needed.
- When two fragments both declare the same array-typed section, hard-error naming both fragments and the section. Never last-wins: a silent pick is a silent wrong answer that surfaces months later as a status legend nobody chose.
- Scalars and tables continue to merge as they do today. The error is specific to arrays, because arrays are the only place the merge cannot express the user's intent.
- The error message names the two sources and suggests the resolution — pick one, or move the difference into `[overlay]`.
- Tests: `[overlay]` lands last and beats every axis fragment; two fragments declaring `status` error and name both; two declaring different sections compose cleanly; scalar and table merges are unchanged; a preset with only an overlay works.
