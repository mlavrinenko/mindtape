// www/utils/slices.typ — the shared half of a docs page whose blocks are
// slices of a checked example fixture.
//
// www/generate-examples.sh writes one fact per fixture under www/generated/
// (its shape: {root, slices: {id: {file, code}}}), and the page declares the
// ids it quotes. `checked-slices` asserts the two sets match EXACTLY, in both
// directions: an id the fact lost fails the render (the fix the message names
// can be either side), and a slice no longer quoted is caught too — a page
// that quietly stopped showing a form is the drift this whole mechanism
// exists to prevent. `render-slices` builds the annotated block from the same
// fact, so the comment prefix and the code stay one string.

#let checked-slices(fact, name, expected) = {
  let got = fact.slices.keys()
  if not (
    expected.all(id => id in fact.slices) and got.len() == expected.len()
  ) {
    panic(
      "www/generated/"
        + name
        + ".json slices ("
        + got.join(", ")
        + ") do not match this page's list ("
        + expected.join(", ")
        + "); regenerate the docs and fix whichever side moved",
    )
  }
  fact.slices
}

// Each part is (slice id, one-line note), and the note lands as the `//`
// comment above the real lines — so the block reads as an annotated file and
// every code line in it is still the fixture's own.
#let render-slices(slices, parts) = (
  parts
    .map(p => {
      let s = slices.at(p.at(0))
      "// " + s.file + " — " + p.at(1) + "\n" + s.code
    })
    .join("\n\n")
)
