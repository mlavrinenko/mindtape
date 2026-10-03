// docs:slice task
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix the login redirect",
  status: wip(2026, 6, 14)[reproduced on staging], // the status, the date it was set, and a note
  priority: ice(impact: 8, confidence: 0.7, ease: 6), // a framework call or a number; difficulty too
  estimate: (days: 1, hours: 2), // units the project allows
  assignee: "ada", // who is working on it
  links: depends-on("add-a-session-store.typ"), // typed edges; a target is a path from this file
  tags: ("backend",),
  x-speconaut: (spec: "login-flow"), // a tool's own field; mt passes it through
)
// docs:end

= Summary

Send the user back to the page they asked for after they log in.
