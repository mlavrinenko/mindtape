#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Stop just fork from warning that it cannot cd",
  status: done(2026, 10, 13),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A recipe cannot move its caller's shell, so wt printed three lines of shell-integration advice on every fork
