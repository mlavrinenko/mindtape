#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "init presets",
  priority: ice(
    impact: 9,
    confidence: 0.75,
    ease: 3,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Replace the two-mode `mt init` (12-line slim, 211-line --full) with named presets over a set of independent configuration axes, plus a wizard that walks those same axes. Research across CLI task trackers found no sweet spot on config size — todo.txt keeps users because it has no surface and Taskwarrior keeps users because it has a large one — so the tool should ship a few coherent shapes rather than pick one. A preset is what makes a project self-contained: a global defaults.toml layer cannot be relied on by anyone who clones the repository, so a users taste has to land as real content inside the project config, and the preset is what stamps it there.

== Scope

- Axes, each one question and one config fragment: workflow (status legend plus transitions), priority, estimate, required fields, organisation (derive-tags), body headers, recurrence.
- Built-in presets, four, none named for a team size or an audience: `todone` (todo to done, floor arrays emptied), `flow` (reviewed lifecycle with enforced transitions), `personal` (hide-until dates, contexts as folders, completion-anchored recurrence, a review view), `gated` (required spec sections plus a review gate; done unreachable from wip).
- Preset naming rule: name the opinion, never the position. No preset called default, base or standard — Renovate renamed config:base to config:recommended precisely because a neutral-sounding name hid a subjective choice.
- Compose at init time, flatten to one readable file. When two fragments both declare the same array-typed section (status, priority.bands, difficulty.anchors, views, roots) the composer hard-errors naming both fragments. Never last-wins: arrays replace wholesale in the merge, so a silent pick would be a silent wrong answer.
- Split content from commentary: `--preset` picks content, `--annotate` renders the same content with the explanatory comments. The 211-line artifact becomes a rendering, not a mode.
- Wizard, TTY-gated: `mt init` bare with a terminal walks the axes; `--preset X` never prompts; `--no-input` and a non-terminal stdin fall through to a named default. Every question has a corresponding flag. Flags must genuinely suppress the questions they answer — npm, create-next-app, create-vite and create-astro all shipped that bug.
- Answers are recorded as a comment block in the written config, so the project documents how it was made.
- User presets at `$XDG_CONFIG_HOME/mindtape/presets/<name>.toml` or `<name>/` with `config.toml` plus `templates/`. One file or directory per preset — Vue CLI put every preset in one dotfile and a single syntax error broke scaffolding entirely.
- Resolution order: user preset, then built-in, then an explicit path when the argument has a separator or a .toml suffix, then error listing known names. Never fall through to a network fetch on an unknown name, which is how cargo-generate turns a typo into a git clone. User presets shadow built-ins, the ordering Vue CLI had to patch in.
- `mt preset save <name>` dumps the current project config as a raw overlay plus its templates. No attempt to reverse-engineer answers.
- `[overlay]` in a preset carries raw config for anything the axes cannot express, merged last.
- Presets bundle templates, always. A preset that sets a strict framework priority without a matching template produces a project whose first `mt add` fails, because `required_core_slots` cannot splice a framework that has no scalar form.
- Remote presets are out of scope. A preset carries Typst that gets evaluated, and the field record is bad — cookiecutter ships unsandboxed hooks by design and carried a 9.8 command injection, git carried a 9.0 hook-execution-during-clone, npm `create-*` typosquats have exfiltrated CI secrets. If added later, copy cargo-generate explicit consent or direnv content-hash allow.
- No preset ships a closed `[tasks.tags] allowed` list. Predeclared vocabularies are a documented reason people leave one tracker for another.
- Preset matrix test, the gate that makes first run trustworthy: for every built-in preset and every reachable wizard answer set, init then add with only a title then check then flip to terminal then check, all green.
