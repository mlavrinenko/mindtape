#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Post standup notes",
  status: todo(2026, 9, 15),
  recur: every(days: 1),
  // docs:slice recurred-from
  links: recurred-from("standup-notes-2026-09-14.typ"),
  // docs:end
)

= Summary

What changed, what is blocked, and who needs a decision today.
