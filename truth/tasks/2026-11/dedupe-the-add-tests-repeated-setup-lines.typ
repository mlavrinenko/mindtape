#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Dedupe the add tests' repeated setup lines",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  kind: "dev",
  tags: ("stateless", "tests", "repeats"),
  assignee: "agent",
  status: done(
    2026,
    11,
    24,
  )[the add tests share support\_tests.rs. 244 add tests pass and just check is green],
)

== AS-IS

The `add` tests paste their setup lines, and the repeats arm files five findings against them.

+ Run `jscpd --no-tips -f rust -k 15 -l 1 -r json -o /tmp/rep crates`, then the `score_repeats` jq from `contrib/ask-refactoring/repeats.sh`.
+ 🔴 The arm reports 28 snippets at 15 tokens with gain >= 300, and five anchor in `crates/mindtape-cli/src/cmd/add/` tests:
  - `pipeline_tests.rs:85`: 132 copies of a `create_dir_all` setup (44 tokens, gain 5094).
  - `pipeline_tests.rs:23`: 28 copies of `const TEMPLATE` (22 tokens, gain 444).
  - `dedup_tests.rs:97`: 71 copies of `merge_layers_with_packs_using` (32 tokens, gain 1875).
  - `dedup_tests.rs:68`: 23 copies of `plan_next` (43 tokens, gain 821).
  - `field_values_tests.rs:3`: 22 copies of the file header (23 tokens, gain 363).

== TO-BE

- 🟢 A `support_tests.rs` sibling holds the fixtures the add tests share: the template sources, the seed maps, the scaffold and parse helpers, the values-file and editor-script writers.
- 🟢 Every copy calls the helper. A fixture change lands once.
- 🟢 The tests keep their behavior: `just test` passes with the same assertions.

```
$ just test
    Starting 1 test across 1 binary
    ...
     Summary [  12.345s] 1234 tests run: 1234 passed, 0 skipped
```

== Changes

- `crates/mindtape-cli/src/cmd/add/support_tests.rs`: the shared fixtures.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: declare the test sibling.
- `crates/mindtape-cli/src/cmd/add/scaffold_tests.rs`, `values_tests.rs`, `values_tests_ops.rs`, `values_tests_templates.rs`, `yaml_values_tests.rs`, `tokens_tests.rs`, `interactive_tests.rs`, `interactive_tests_editor.rs`, `pipeline_tests.rs`, `dedup_tests.rs`, `field_values_tests.rs`, `hooks_tests.rs`, `notes_tests.rs`, `draft_tests.rs`, `output_tests.rs`, `tests.rs`, `tests_subprocess.rs`: call the helpers.

== Out of scope

- The first-line `//!` convention: every file keeps its module doc.
- The repeats gate and its threshold: the arm stays as it is.
- The qahq bump.
- The copies outside `crates/mindtape-cli/src/cmd/add/`: other areas own them.

== Dogfood

The add tests read as one fixture per setup, so a change to a template or a values file lands once and the suite still pins the same behavior.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[one helper per fixture means a change lands once and the suite still proves the same behavior].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
