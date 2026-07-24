#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Analyze chord progressions in the recital repertoire",
  priority: ice(
    impact: 10,
    confidence: 0.95,
    ease: 8,
  ),
  tags: ("theory-drill",),
  links: parent("circle-of-fifths-drill.typ")
    + related(
      "interval-recognition-ear-training.typ",
    )[leans on the same interval vocabulary],
  due: on(2026, 7, 10),
  status: proposed(2026, 7, 3),
)

== Notes

Roman-numeral the harmonic progressions under Für Elise and Clair de Lune —
seeing the ii-V-I and borrowed chords by name should make the memorization
stick instead of being pure muscle memory.
