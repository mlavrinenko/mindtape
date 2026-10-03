#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Expire idle sessions",
  status: wip(2026, 9, 28),
  priority: 8,
  due: datetime(year: 2026, month: 10, day: 15),
  assignee: "ada",
  tags: ("backend", "security"),
  links: depends-on("pick-a-session-store.typ"),
)

== Problem

A session stays valid until logout, however long it sits idle.
