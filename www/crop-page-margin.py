"""www/crop-page-margin.py — trim the page margin around the hero's task SVG.

www/generate.sh compiles the hero task with `typst compile`. That render
carries the page margin the `task` show rule sets (lib/prelude.typ,
`_render-task`), and the docs card frames it again with its own border. The two
frames would show as one 80px gutter, so this script crops a little over half
the page margin from every side. The card's border then frames a thinner page
edge, and every gutter lands under half its uncropped width.

The crop is a docs preview choice, not a change to the typeset page: compiling
the task still produces the full page. Typst translates the page content by its
margin in the SVG's first `typst-group` transform, so the script reads the
margin from the render instead of repeating the prelude's numbers. It rewrites
only the root `viewBox` and `width`/`height`, and fails the build if the SVG is
not the one-page `typst-doc` it expects.
"""

import re
import sys
from pathlib import Path

# Cropped from each side, as a share of the page's left margin. Slightly over
# half because the bottom margin is 0.1cm wider than the sides, and it must
# still land under half its own width.
CROP_SHARE = 0.55

ROOT = re.compile(
    r'<svg class="typst-doc" viewBox="0 0 (?P<width>[\d.]+) (?P<height>[\d.]+)" '
    r'width="[\d.]+pt" height="[\d.]+pt"'
)
PAGE = re.compile(
    r'<g class="typst-group" transform="matrix\(1 0 0 1 (?P<x>[\d.]+) [\d.]+\)"'
)


def crop(svg):
    root = ROOT.search(svg)
    page = PAGE.search(svg)
    if root is None or page is None:
        sys.exit("crop-page-margin: not a one-page typst-doc SVG")
    width = float(root["width"])
    height = float(root["height"])
    inset = float(page["x"]) * CROP_SHARE
    if width <= 2 * inset or height <= 2 * inset:
        sys.exit("crop-page-margin: the page margin is too large to crop")
    cropped = (
        f'<svg class="typst-doc" viewBox="{inset:.17g} {inset:.17g} '
        f'{width - 2 * inset:.17g} {height - 2 * inset:.17g}" '
        f'width="{width - 2 * inset:.17g}pt" height="{height - 2 * inset:.17g}pt"'
    )
    return svg[: root.start()] + cropped + svg[root.end() :]


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: crop-page-margin.py <file.svg>")
    path = Path(sys.argv[1])
    cropped = crop(path.read_text(encoding="utf-8"))
    tmp = path.with_name(path.name + ".crop")
    tmp.write_text(cropped, encoding="utf-8")
    tmp.replace(path)


if __name__ == "__main__":
    main()
