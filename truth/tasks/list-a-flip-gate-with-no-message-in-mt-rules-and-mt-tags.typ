#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "List a flip gate with no message in mt explain rules and mt explain tags",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a gate a flip enforces is visible in mt explain rules and mt explain tags],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 4),
  links: edge(
    "surfaced-by",
    "read-has-tags-and-spaced-has-tag-calls-in-mt-tags.typ",
  ),
)

== AS-IS

+ Explaining the out-of-scope note in `read-has-tags-and-spaced-has-tag-calls-in-mt-tags.typ` after fixing `mt explain tags`.

+ 🔴 #raw("crates/mindtape/src/config/active_rules.rs"): `active_rules` skips every `[[on.flip]]` action with no `message`, so a gate that refuses a flip is absent from `mt explain rules --for flip` and `mt explain tags --for flip`.


With `ship` read by a run-less gate with no message and `cli` by one with a message, only the second lists:

```
$ mt explain rules --for flip
error  on.flip[1].actions[0]  cli gate
$ mt explain tags --for flip
cli  on.flip[1].actions[0]
     cli gate
```

The first still refuses at flip time, through `runless_gate` with `DEFAULT_ADVICE`:

```
crates/mindtape/src/config/active_rules.rs:
    let Some(message) = &action.message else {
        continue;
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads.

== TO-BE

- 🟢 `mt explain rules --for flip` lists every action with `gate = true`, message or not, as `(no message set)` when it has none.
- 🟢 `mt explain tags --for flip` names the tags that gate's `when` reads.
- 🟢 `mt explain rules --for flip --json` keeps `"message": null` for a gate with no message.

```
$ mt explain rules --for flip
error  on.flip[0].actions[0]  (no message set)
error  on.flip[1].actions[0]  cli gate
$ mt explain tags --for flip
ship  on.flip[0].actions[0]
      (no message set)
cli   on.flip[1].actions[0]
      cli gate
```


== Changes

- `crates/mindtape/src/config/active_rules.rs`: list an action when `gate = true` or it carries a `message`, and update the module, `ActiveRule.message`, `active_rules` and `flip_actions` docs that say only message-carrying actions list.
- `crates/mindtape/src/config/active_rules_tests.rs`: rename and re-document the existing test, then pin a run-less gate and a gating `run` with no message, while a message-less `show` and non-gating `run` stay out.
- `crates/mindtape-cli/src/cmd/explain/tags_tests.rs`: pin that `--for flip` lists the tag read by a message-less gate. The rules table's `(no message set)` render is already covered.
- `www/content/cli/explain.typ`: say a flip gate lists with or without a `message`, and another action lists only when it carries one; say a tag's rule shows the message `mt explain rules` gives it, `(no message set)` included.


== Out of scope

- The text `mt explain rules` prints for a rule with no message. It stays `(no message set)`, as a body rule without one already shows.
- The `[[on.flip]]` refusal text in `cmd/flip/hooks.rs`. A run-less gate keeps `DEFAULT_ADVICE`, and a gating `run` with no message keeps its current text.


== Dogfood

Any project that writes `gate = true` without a `message` sees it in `mt explain rules --for flip` and `mt explain tags --for flip`. This repo's shipped flow gate carries a message, so its own listing does not change.
