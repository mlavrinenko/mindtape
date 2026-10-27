#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Route every path mt prints through one helper",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli", "mt"),
  estimate: (src-files: 40),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the Windows test job passes, so a Windows user gets the paths the docs show],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a clippy guard refuses an ad hoc `Path::display` in output, so a new message can't mix separators],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
  links: depends-on(
    "show-paths-in-the-native-separator-with-a-path-separator-override.typ",
  ),
)

== Problem

+ Push a branch and run the `windows` job of `.github/workflows/ci.yml`.
+ 🔴 88 tests fail, such as `mv_e2e::mv_renames_the_file_and_retargets_every_inbound_edge`, which reads `moved tasks\old.typ -> tasks\new.typ` where it expects `/`.


== Environment

The `windows-latest` runner of the `Test (Windows)` CI job, run 40594914164.


== TO-BE

- 🟢 Every path in a message, a note, a dry-run plan or an error place goes through `mindtape::display_path`, and every path in JSON, CSV or a file goes through `mindtape::slash_path`.
- 🟢 A clippy `disallowed-methods` entry refuses `Path::display` outside tests, naming the two helpers.
- 🟢 Each failing Windows test expects what the rule says: native for people, `/` for programs.


== Changes

- `clippy.toml`: refuse `std::path::Path::display`.
- `crates/mindtape-cli/src/diag/mod.rs`, `json.rs`: keep the site as a path, render the place natively and the JSON `file` and `place` with `/`.
- `crates/mindtape/src/model/canonical.rs`, `crates/mindtape-cli/src/render/output.rs`: the JSON and CSV `file` with `/`.
- `crates/mindtape/src/loader/errors.rs`, `validate/violation.rs`, `validate/links.rs`, `config/*.rs`: library places and messages.
- `crates/mindtape-cli/src/cmd/**`: each message's paths.
- `crates/mindtape-cli/src/diag/render.rs`: leave `\` unquoted in a Windows command hint.
- `crates/mindtape-cli/tests/it/support/mod.rs`: a `native` helper for test expectations.


== Out of scope

- `init_e2e::every_preset_supports_add_check_and_a_full_walk`, which fails on a `sh` gate under `cmd`, not on a separator.


== Dogfood

Not applicable: this repo is developed on Linux, where both helpers print `/`.

