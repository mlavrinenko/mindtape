#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept links slot values in mt add",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "stateless"),
  estimate: (src-files: 5),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[an agent files a linked task from one YAML value, with no second command],
      ),
      ..(
        "readable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the YAML and editor paths feed the same LinkValue shape the library renders],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 4),
  links: depends-on("add-a-links-slot-kind-to-the-template-engine.typ"),
)

== AS-IS

+ Run `mt add -f -` against a template with the `links` slot kind the previous task adds.
+ 🔴 The library renders the slot, but `mt add` cannot fill it: `yaml_values` rejects a sequence of mappings, `--set` turns the value into one string, and the editor scaffold has no shape for it.

```
$ mt add -t probe -f - <<'YAML'
links:
  - kind: surfaced-by
    task: origin.typ
YAML
error: nested mappings are unsupported: 'links'
```

== TO-BE

- 🟢 `-f` and `--edit` read a YAML or JSON list of `{kind, task, note}` mappings into `RawValue::Links`.
- 🟢 An empty list is an empty links value, so an optional slot files no edges.
- 🟢 A missing `kind` or `task`, an unknown key, or a mixed list fails naming the slot and the bad entry.
- 🟢 `--set links=...` fails with a hint to use `-f`, since one token cannot carry the list.
- 🟢 The editor scaffold writes a links slot as a YAML list and parses it back.

```
$ mt add -t probe -f - <<'YAML'
title: Probe
links: [{kind: surfaced-by, task: origin.typ}]
YAML
wrote tasks/probe.typ
```

== Changes

- `crates/mindtape-cli/src/cmd/add/values_tests.rs`: the red test first, a YAML and a JSON list of mappings become `RawValue::Links`.
- `crates/mindtape-cli/src/cmd/add/yaml_values.rs`: parse a sequence of link mappings, with `kind` and `task` required and `note` optional.
- `crates/mindtape-cli/src/cmd/add/values.rs`: the JSON counterpart.
- `crates/mindtape-cli/src/cmd/add/interactive.rs`: the scaffold and seed for a links slot.
- `crates/mindtape-cli/src/diag/from_lib.rs`: the type error hints at `-f`.
- `skills/mindtape/references/add-slots.md`: the YAML shape and the `--set` refusal.

== Alternatives

- A `list` slot over `kind|target|note` strings: it needs a second grammar and cannot carry a note that holds the separator.
- A `--link` flag on `mt add`: it duplicates `mt link` and cannot feed a template's own slot choice.

== Out of scope

- The library-side slot kind, which the previous task lands.
- The finding template, which the next task fills with this kind.

== Dogfood

The finding template's `links` slot is the first real use, and `mt add -f -` files a finding with its origin in one command.
