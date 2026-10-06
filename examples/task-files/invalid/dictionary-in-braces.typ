#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix the login redirect",
  status: todo(2026, 7, 1),
  // docs:slice dictionary-invalid
  estimate: {days: 1, hours: 2},
  // docs:end
)
