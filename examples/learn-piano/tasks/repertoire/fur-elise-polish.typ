#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Für Elise — polish for recital",
  priority: ice(
    impact: 9,
    confidence: 0.85,
    ease: 6,
  ),
  estimate: (hours: 3),
  tags: ("classical", "recital"),
  links: depends-on("../technique/major-scales-two-octaves.typ"),
  due: on(2026, 7, 15),
  start: on(2026, 6, 15),
  status: wip(2026, 7, 4)[A section memorized, working the C section runs],
)

== Notes

Main theme and A section are recital-ready; the C section's right-hand
sixteenth-note run still rushes. Ready to start: the major-scales technique it
leans on is already done.
