#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept external refs in the commit trailer",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "gating", "stateless"),
  estimate: (src-files: 5),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a repo that tracks work elsewhere keeps `Refs:` as its one trailer and files no mirror task],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the schema and commit-message docs state the external patterns and the accepted separators],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 2),
)

== AS-IS

+ A repo tracks work outside mindtape, such as Redmine issue `r#45782`.
+ It sets `[commits] trailer = "Refs"` and writes `Refs: r#45782`.
+ 🔴 `mt check --commit-msg` refuses the message, and no config key says the value is an external ref.

```
$ mt check --commit-msg msg.txt
error: msg.txt:3: Refs: `r#45782` names no task
```


== TO-BE

- 🟢 `[commits] external-refs` takes regexes. A token that names a task is a task ref; else a token matching one of the regexes is an external ref; else it is refused as today.
- 🟢 A commit may name both kinds, and `has(refs)` is true when it names either.
- 🟢 One trailer line may hold several refs split on commas or whitespace; repeated `Refs:` lines work too.
- 🟢 `touches()` indexes task refs only, and a `scope = "task"` rule skips external refs.

```toml
[commits]
trailer = "Refs"
external-refs = ["^r#[0-9]+$", "^[A-Z][A-Z0-9]+-[0-9]+$"]
```

```text
Refs: redesign r#45782
Refs: redesign, r#45782
Refs: redesign
Refs: r#45782
```


== Changes

- `crates/mindtape/src/commit/tests.rs`: red tests first for a stem, an external ref, both, comma, space and repeated lines.
- `crates/mindtape/src/config/commits.rs`: `external-refs`, its schema description and load-time regex validation.
- `crates/mindtape/src/commit/mod.rs`: split each trailer value, classify every token, and keep external refs out of the task lookup.
- `crates/mindtape/src/commit/refs.rs`: `touches()` splits the same way and indexes task refs only.
- `crates/mindtape-cli/tests/it/commit_msg_e2e.rs`: the end-to-end check for all three forms.
- `www/content/config.typ`, `docs/src/contributing/commit-messages.typ`, `schema/config.schema.json`, `CHANGELOG.md`: the key, the forms and the one-line escape for a ref containing a separator.


== Alternatives

- A separate `External-Refs:` trailer: it forces a second header and breaks the one-trailer habit.
- Split on `;`, `/`, `&` and `and` like Python Semantic Release: those characters occur in URLs and paths, so the split would corrupt refs.
- Accept any unknown value: a typo in a task stem would pass, and the rule that a commit names its work would stop meaning anything.


== Out of scope

- Closing keywords such as `Closes` or `Fixes`; mindtape reads only its configured trailer.
- Resolving an external ref against its tracker; the value stays opaque.
- Separator characters beyond comma and whitespace.


== Dogfood

This repo's `[commits] external-refs` stays empty, so every `Refs:` still names a task. The docs example shows a repo that adds `^r#[0-9]+$`.

