#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fix the Windows path expectations in the new warning and pin tests",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 1,
  tags: ("tests", "stateless", "mt"),
  estimate: (src-files: 0),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the Windows job holds the tests to the separator the code prints],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 3),
)

== AS-IS

+ Run the Windows test job on `d12bc326`.
+ 🔴 Four tests fail because they compare native Windows paths with `/`:

```
warnings::tests::a_single_warning_keeps_its_place_without_a_count
  left: "warning: tasks\\a.typ:6:1: no text within stars\\nhint: hint\\n"
 right: "warning: tasks/a.typ:6:1: no text within stars\\nhint: hint\\n"
loader::tests_pins::a_pin_with_no_manifest_still_names_its_own_path
  the pin's manifest is named: ...\\lib\\typst.toml
```


== Environment

- Windows CI job `125596168262` on commit `d12bc326`, Rust 1.89.


== TO-BE

- 🟢 The warning and pin tests ask `display_path` for the expected separator.
- 🟢 The Windows job passes without changing what `mt` prints on Windows.

```
$ cargo test --workspace
test result: ok
```


== Changes

- `crates/mindtape-cli/src/warnings_tests.rs`: build each expected path through `mindtape::display_path`.
- `crates/mindtape/src/loader/tests_pins.rs`: assert the pin's manifest through `display_path` too.


== Out of scope

- `display_path` itself. It already prints the native separator, which the Windows job proves.


== Dogfood

The release's Windows job runs these tests on every future tag.

