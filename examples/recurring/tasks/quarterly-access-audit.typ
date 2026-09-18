#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Review who can reach production",
  status: todo(2026, 9, 14),
  priority: 7,
  tags: ("security",),
  due: on(2026, 10, 1),
  // docs:slice every-policy
  recur: every(
    months: 3,
    anchor: "due",
    until: on(2027, 1, 1),
    count: 4,
    materialize: "mutate",
  ),
  // docs:end
)

= Summary

Walk the access list, drop what the last quarter made unnecessary, and keep
the evidence in the audit folder.
