#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Re-record the version groups in the release hook",
  status: done(2026, 10, 29),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

`just release 0.3.1` stopped at its own `just check`: the bump changes the Cargo manifests, the sources of the `release-notes` and `cli-name` outdatty groups, and nothing re-recorded them.
