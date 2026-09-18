#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rotate the TLS certificate",
  status: todo(2026, 9, 14),
  priority: 8,
  tags: ("infra",),
  due: on(2027, 1, 5),
  // docs:slice every-years
  recur: every(years: 1, anchor: "due"),
  // docs:end
)

= Summary

Issue and install the new certificate before the current one expires.
