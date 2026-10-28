#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Quiet the check-windows recipe's cargo output",
  status: done(2026, 10, 28),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

just check-windows printed a Checking line per crate, about 200 lines, to say it passed
