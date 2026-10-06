#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix the login redirect",
  status: todo(2026, 7, 1),
  // docs:slice links-invalid
  links: depends-on("add-a-session-store.typ"),
    related("log-failed-logins.typ")[same handler],
  // docs:end
)
