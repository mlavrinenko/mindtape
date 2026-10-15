#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Vendor the CSV parser",
  // docs:slice failed-note
  status: failed(2026, 6, 30)[its license forbids vendoring],
  // docs:end
  priority: 3,
)

= Summary

Copy the parser into the tree so a release never waits on upstream.
