// docs:slice task
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix the login redirect",
  status: wip(2026, 6, 14)[reproduced on staging], // (name, date, note); note is a trailing block
  priority: ice(impact: 8, confidence: 0.7, ease: 6), // or a raw int; difficulty too
  estimate: (days: 1, hours: 2), // opaque unit dict; allowed units per project
  assignee: "ada", // actor working it; config-bounded, not ownership
  links: depends-on("add-a-session-store.typ"), // typed edges by file stem
  tags: ("backend",),
  x-speconaut: (spec: "login-flow"), // opaque extension slot (x-<ns>)
)
// docs:end

= Summary

Send the user back to the page they asked for after they log in.
