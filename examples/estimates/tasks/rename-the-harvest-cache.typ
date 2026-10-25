#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rename the harvest cache",
  status: todo(2026, 9, 1),
  priority: 4,
  difficulty: 3,
  // docs:slice count
  // Two new files, plus every file that names the type today.
  estimate: (files: count(2, grep(`HarvestCache`))),
  // docs:end
)

= Summary

Rename `HarvestCache` to `TaskCache`, and split its disk format into two new
files.
