// The board `mt` ships, as the file `mt` reads: the body of
// crates/mindtape/src/at/board.toml, cut out by its `docs:slice` markers and
// written to www/generated/shipped-board-config.json by
// www/generate-examples.sh. /read-many-projects/ and /cli/board/ both render
// it, so a changed default changes both pages in the same build.

#import "slices.typ": checked-slices, render-slices

#let shipped-board = {
  let slices = checked-slices(
    json("../generated/shipped-board-config.json"),
    "shipped-board-config",
    ("board",),
  )
  raw(
    render-slices(
      slices,
      (("board", "the board mt ships"),),
      comment: "#",
    ),
    lang: "toml",
    block: true,
  )
}
