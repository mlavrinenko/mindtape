#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Learn Clair de Lune's arpeggiated middle section",
  priority: ice(
    impact: 9,
    confidence: 0.85,
    ease: 5,
  ),
  tags: ("classical",),
  links: depends-on("../technique/minor-scales-two-octaves.typ"),
  due: on(2026, 8, 1),
  status: blocked(
    2025,
    11,
    10,
  )[waiting on left-hand arpeggio control from minor scales],
)

== Notes

The rolling arpeggiated middle section needs the left hand to hold a steady
arpeggio shape while the right hand floats the melody on top — parked until
minor-scales-two-octaves is solid enough to lean on.
