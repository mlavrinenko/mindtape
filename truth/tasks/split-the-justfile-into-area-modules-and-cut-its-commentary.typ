#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split the justfile into area modules and cut its commentary",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 40),
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
      sound-codebase: helps[a recipe is found by area, and no recipe outgrows ten lines],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 18),
)

== Problem

+ Open `Justfile`: 984 lines, 60 recipes at the top level beside one `docs` module.
+ 🔴 Most recipes carry a comment block that repeats their `[doc]` or points at the script the body already names, and `release`, `mutants::status` and `lint::dry` hold 20 to 40 lines of script inline.


== TO-BE

- 🟢 The root holds the daily verbs. Area recipes live in `docs`, `lint`, `meta`, `crap`, `mutants`, `outdatty`, `bench` and `install` modules under `.just/`.
- 🟢 A recipe holds at most ten lines of comment and body. A longer body is a script under `.just/scripts/`, and a comment says only what a caller cannot infer.


== Changes

- `Justfile`, `.just/*.just`: the modules, the trimmed comments, the extracted bodies.
- `.just/scripts/`: the extracted bodies, and the recipe names the gate scripts hardcode.
- `.mmz/config.yaml`, `outdatty.yaml`: the renamed rules, the new scripts' scopes and sources.
- `.github/workflows/`, `flake.nix`, `.config/wt.toml`, `.githooks/`, `contrib/`: the renamed invocations.
- `docs/src/`, `www/`: the renamed `#just.<name>()` calls, then the regenerated Markdown.


== Rollback

- Revert the commits. Nothing outside the dev tooling changes.


== Out of scope

- The commentary inside `.just/scripts/*.sh`, `.mmz/config.yaml` and `outdatty.yaml`.
- Done tasks that name an old recipe: they record what was true then.


== Dogfood

This repo's contributors run `just lint dry`, `just mutants diff` and `just outdatty update <group>`.

