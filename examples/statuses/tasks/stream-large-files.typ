#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Stream large files",
  // docs:slice custom-status
  status: status("on-hold", 2026, 7, 2, note: [waiting on the storage quota]),
  // docs:end
  priority: 5,
)

= Summary

Import a file larger than memory without loading it whole.
