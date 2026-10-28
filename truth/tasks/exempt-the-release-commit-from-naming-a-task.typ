#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Exempt the release commit from naming a task",
  status: done(2026, 10, 28),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

just release's own chore(release) commit dates CHANGELOG.md and works on no task, so work-names-a-task refused it and stopped the first run of the recipe
