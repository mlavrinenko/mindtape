---
name: Mindtape Documentation
description: The docs as a transit information system, one line of stops from install to internals.
colors:
  paper: "#f5f7f2"
  paper-sunk: "#edf1eb"
  ink: "#101512"
  ink-soft: "#34423a"
  ink-muted: "#65746a"
  rule: "#ccd6ce"
  rule-strong: "#101512"
  green: "#12805a"
  green-deep: "#0a5a3e"
  green-wash: "#e1efe8"
  status-blue: "#2f5f78"
  status-amber: "#9a6200"
  status-red: "#b3261e"
typography:
  display:
    fontFamily: "Archivo, Helvetica Neue, Arial, sans-serif"
    fontSize: "clamp(72px, 7.6vw, 107px)"
    fontWeight: 800
    lineHeight: 0.84
    letterSpacing: "-0.075em"
  headline:
    fontFamily: "Archivo, Helvetica Neue, Arial, sans-serif"
    fontSize: "clamp(46px, 5vw, 62px)"
    fontWeight: 800
    lineHeight: 0.96
    letterSpacing: "-0.055em"
  title:
    fontFamily: "Archivo, Helvetica Neue, Arial, sans-serif"
    fontSize: "30px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "-0.035em"
  body:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "18px"
    fontWeight: 400
    lineHeight: 1.65
  prose:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "16.5px"
    fontWeight: 400
    lineHeight: 1.68
  label:
    fontFamily: "JetBrains Mono, ui-monospace, SFMono-Regular, monospace"
    fontSize: "11px"
    fontWeight: 500
    letterSpacing: "0.1em"
rounded:
  square: "1px"
  dot: "50%"
spacing:
  gutter: "40px"
  sidebar: "232px"
  rail: "210px"
  article-measure: "690px"
  text-measure: "46rem"
components:
  button-primary:
    backgroundColor: "{colors.green}"
    textColor: "#ffffff"
    rounded: "{rounded.square}"
    padding: "0 20px"
    height: "46px"
  button-primary-hover:
    backgroundColor: "{colors.green-deep}"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    padding: "0 20px"
    height: "46px"
  search-field:
    backgroundColor: "{colors.paper-sunk}"
    textColor: "{colors.ink-muted}"
    rounded: "{rounded.square}"
    height: "36px"
  code-panel:
    backgroundColor: "{colors.paper-sunk}"
    textColor: "{colors.ink}"
---

# Design System: Mindtape Documentation

## Overview

**Creative North Star: "The Line"**

The documentation is a transit information system. Every page is a stop on one line, its position is drawn in the left rail, and the current page is marked by a filled stop on the line map. The world is Swiss wayfinding and a paper timetable, not a terminal costume: flat unprinted paper, black rules, one green line, and real command output set as the timetable data.

The system is strict because the product is. A 1360px wrap holds a fixed 232px line map and a fluid reading column; hairlines separate regions instead of cards; the display face is heavy and tight so a page title reads across a room. Code sits on paper in light mode and on a raised night panel in dark mode, with Typst's baked syntax palette remapped for the dark ground. The voice is plain and technical: explain the mechanism, show a real command, move on. A dot exists only where a real destination sits.

**Key Characteristics:**
- One line from masthead to footer, with real stops and a marked current position.
- Black ink, black rules, unprinted paper; green only as the line, the mark, and the primary action.
- Station-sign display type over a serif reading column and monospaced data.
- Squared corners (1px), 1px rules, no shadows, no cards.
- Status is a word and a mark before it is a color.

## Colors

A paper-and-ink palette with one signal green. The green is the logo's own accent, kept across the redesign, and it carries position rather than decoration.

### Primary
- **Signal Green** (#12805a): the line, the active stop, the brand mark, the primary button, and link emphasis. In dark mode it lifts to **Signal Mint** (#45d0a0).
- **Deep Green** (#0a5a3e): hover and pressed states for green controls, and link text on light paper.

### Neutral
- **Paper** (#f5f7f2): the page ground and every flat surface.
- **Paper Sunk** (#edf1eb): inputs, code headers, muted panels; one step down, never a card.
- **Ink** (#101512): display text, rules, panel headers, and the strongest dividers.
- **Ink Soft** (#34423a): secondary prose and rail labels.
- **Ink Muted** (#65746a): metadata, captions, timestamps, and placeholders.
- **Rule** (#ccd6ce): hairline separators inside a region.
- **Rule Strong** (#101512): the border between major regions.
- **Status Blue** (#2f5f78), **Status Amber** (#9a6200), **Status Red** (#b3261e): work, blocked, and refused states, always with a word and a mark, never alone.

**The One Line Rule.** One continuous green line carries position through the page. There is no second accent color, no gradient, and no colored panel.

**The State Before Hue Rule.** A status is a glyph and a word first. Color repeats the meaning; it never carries it alone.

## Typography

**Display Font:** Archivo (with Helvetica Neue, Arial)
**Body Font:** Literata (with Georgia, serif)
**Label/Mono Font:** JetBrains Mono (with ui-monospace, SFMono-Regular)

**Character:** A wide, heavy grotesque for station names, a book serif for reading, and a neutral mono for data, all self-hosted variable Latin subsets under `assets/fonts/` with their SIL OFL licenses. No third-party font requests.

### Hierarchy
- **Display** (800, clamp(72px, 7.6vw, 107px), 0.84, -0.075em): the home page title only, set across the full hero width with the cassette mark at its right edge.
- **Headline** (800, clamp(46px, 5vw, 62px), 0.96, -0.055em): article titles. Distinct from display so a long page name still fits.
- **Title** (700, 30px, 1.1, -0.035em): section heads on the home page and inside prose. H2s inside prose carry a hairline above them; the first one does not.
- **Lede** (400, 21px, 1.42): the home summary in Literata; article ledes drop to 19px/1.5 and cap at 60ch.
- **Body** (400, 18px, 1.65): default prose. Article prose reads at 16.5px/1.68 in a 690px column.
- **Label** (500, 11px, 0.08 to 0.16em, uppercase): station numbers, route labels, panel headers, and metadata. Uppercase comes from CSS, never from the source string.
- **Code** (400, 12.5px, 1.62): panels on the home page; 11.5px inside article prose.

**The Measured Column Rule.** Reading text stays between 60 and 72 characters. Display type may exceed the column, reading text may not. **The Legibility Floor.** No type role is set below 11px; labels separate themselves by family, weight, tracking and color, and inline code keeps its parent's size.

## Layout

The page is a single 1360px wrap with 40px gutters. Below the masthead and route strip, a two-column shell holds a fixed 232px line map and a fluid main column, separated by a gap that scales from 1.5rem to 3.5rem. Articles add a second, sticky rail of 210px for calling points, with a fixed 56px gap; the prose itself stops at 690px so the rail never crowds the measure. The home page uses the full main column for the hero and caps each section at 690px, except the rules table, which is allowed to run wide.

Vertical rhythm comes from hairlines and white space: 42px above the shell, 40px before a section head, 34px after an article head, 24px between a section and its first paragraph. The line map’s numbers (01 to 07) are station numbers that mark a real position; section heads in the reading column carry none. At 52rem the sidebar becomes a drawer and the calling points fold into a details block above the prose; at 40rem the masthead wraps to two rows, the search takes the full width, and the display title steps down to `clamp(64px, 17vw, 92px)`.

## Elevation & Depth

There are no shadows. Depth is drawn with two rules: the 1px hairline inside a region and the 1px strong border between regions. The code panel separates its header with a hairline rather than lifting off the page, and the only dimensional mark is the filled green stop on the rail's own 1px line.

**The Flat Paper Rule.** Surfaces are flat at rest and at every state. If an element needs emphasis, it gains a rule, a weight step, or an inverted header, never a shadow.

## Shapes

Everything is squared. Corners use a 1px radius, which keeps anti-aliasing clean without reading as rounded. Borders are 1px, with 3px reserved for the top edge of a code panel and a callout. Dots are the only circular form: 7px stops on the line map and the calling points rail, and the 8px end-of-line mark in the footer. Buttons, inputs, and code panels all keep the same square silhouette.

**The Squared Rule.** No rounded cards, pills, or floating panels. A circle appears only where a stop on a line is meant.

## Components

### Buttons
- **Shape:** square (1px radius), 46px tall, 20px side padding, 11px uppercase label at 700 with 0.08em tracking.
- **Primary:** signal green ground, white text, a 16px arrow. Hover deepens to deep green.
- **Ghost:** transparent ground, 1px ink border, ink text; hover inverts to ink ground and paper text. Focus is a 2px offset ring in the accent color.

### Search Field
- **Style:** 36px tall, paper-sunk ground, 1px rule, 12px sans placeholder, a 14px drawn magnifier, and a monospace `/` key hint at the right.
- **Behavior:** Pagefind replaces the static placeholder in place. `/` focuses the input from anywhere; Escape closes the result panel.

### Mark
- **Style:** the cassette silhouette from `logo.svg` as a flat CSS mask in signal green. The 26px mark sits beside the masthead name.
- **Hero:** the same mark fills a square the size of the display title at the right edge of the hero's title row. The h1 already names the product, so the hero mark is decorative, and it hides below 40rem.

### Code Panel
- **Style:** 1px strong border with a 3px green top edge. A 36px paper header carries a 7px green square, the language label, the file path in muted mono, and a COPY control. The body is code on light paper in light mode and on a raised night panel (#1a261e) in dark mode.
- **Behavior:** COPY copies the block and says "Copied" for two seconds. Article panels take the language label from the first line or the fence, never a hand-written guess.

### Line Map (signature)
- **Style:** a fixed 232px rail under the word DOCUMENTATION. Groups 01 to 07 hang from a 1px vertical line; each destination is a 7px ring, the current page a filled green stop and a bold label.
- **Behavior:** the rail is sticky at 22px on wide screens; below 52rem it becomes a drawer with the same line and stops.

### Calling Points
- **Style:** a 210px sticky rail under a 1px strong rule. Each entry is a bordered cell on the vertical line, with a 7px stop centred on its label. The current entry is green and bold, every stop above it is filled, and a green segment runs from the list top to the current stop; later stops stay hollow. Narrow screens fold the list into a `<details>` block above the prose.
- **Behavior:** the rail follows the heading under the masthead on scroll and moves at once on a click. A page with fewer than three headings has no rail.

## Do's and Don'ts

- **Do** draw position with the line: a dot only where a real page or group exists.
- **Do** keep the green under about a tenth of any screen. Its rarity is the signal.
- **Do** keep code on a panel: light paper in light mode, a raised night panel in dark mode, with the baked palette remapped.

- **Don't** add a second accent color, a gradient, or a glass surface.
- **Don't** round a card, add a shadow, or float a panel.
- **Don't** use color alone for status, and never pair a status hue with a different word.
- **Don't** hand-type command output: every capture comes from `www/generated/`.
