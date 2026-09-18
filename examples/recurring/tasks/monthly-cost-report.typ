#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Send the cost report",
  status: todo(2026, 9, 14),
  priority: 3,
  tags: ("finance",),
  due: on(2026, 10, 5),
  // docs:slice rrule
  recur: rrule("FREQ=MONTHLY;BYDAY=1MO"),
  // docs:end
)

= Summary

The first Monday report: last month's spend, the outliers, and the forecast.
