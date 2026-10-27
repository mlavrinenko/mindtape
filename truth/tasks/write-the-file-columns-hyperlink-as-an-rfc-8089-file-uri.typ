#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write the file column's hyperlink as an RFC 8089 file URI",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a ctrl-click on a Windows terminal opens the task],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
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

+ On Windows, run `mt ls --hyperlinks always --columns slug,file`.
+ 🔴 The link reads `file://C:\proj\tasks\a.typ`, which names the host `C:`, and a root joined with a `/` path gives `file:///home/user/proj\truth/...`.


== Environment

Windows, any terminal with OSC 8 support, and the `Test (Windows)` CI job.


== TO-BE

- 🟢 `{abs}` in `[ls] hyperlink-target` is what follows `file://` in an RFC 8089 URI: `/C:/proj/tasks/a.typ`, `/home/u/proj/tasks/a.typ`, or `host/share/tasks/a.typ` for a UNC path, with no `\\?\` prefix.
- 🟢 `{abs}` and `{rel}` are `/`-separated and percent-encoded, so a space reads `%20`.
- 🟢 `--path-separator` leaves both alone.


== Changes

- `crates/mindtape-cli/src/render/file_uri.rs` (new), `file_uri_tests.rs`: the red tests first, Windows paths parsed on every OS; the URI path of an absolute and of a relative path.
- `crates/mindtape-cli/src/render/file_path.rs`: substitute the URI forms.
- `crates/mindtape/src/config/ls.rs`: say what `{abs}` and `{rel}` hold.
- `www/content/cli/ls.typ`: the same.


== Out of scope

- Other URI schemes' quirks: a template that needs another form gets its own token when someone asks.


== Dogfood

Not applicable: this repo's `[ls] hyperlinks` stays `never`.

