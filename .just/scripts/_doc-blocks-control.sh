#!/usr/bin/env bash
# .just/scripts/_doc-blocks-control.sh — the control `just check-doc-blocks`
# runs before it scans the real tree. Runs from the repo root and writes only
# to a temp dir, since `just check` runs its arms in parallel.
#
# A scan that finds nothing looks the same as a scanner that reads nothing, for
# example after a typstyle bump changes the `--ast` format. So the control
# scans synthetic files carrying every known-bad shape and every known-good
# one, and requires the exact rows below.
set -eo pipefail

# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh

fail_control() {
    echo "check-doc-blocks: control failed: $*" >&2
    exit 1
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Each line from 2 down is one bad shape, so its line number is its name.
cat >"$tmp/bad.typ" <<'EOF'
= Bad
```sh
mt ls
```
````typ
```nested```
````
#let d = (examples: ```sh
mt add
```,)
#raw("a\nb", lang: "sh", block: true)
#raw(
  "kind = \"" + kind + "\"",
  lang: "toml",
  block: true,
)
#raw(("a", "b").join("\n"), block: true)
#raw("a\n".trim(), block: x)
#let lines = ("a", "b")
#raw(lines.join("\n"), block: true)
#let f(x) = raw(x, block: true)
#f("lit")
#let g(pre, body: "") = { let t = body.trim(); raw(strip(t), block: true) }
#g(1, body: "lit")
#let h(y) = f(y)
#h("lit")
#let code = raw.with(lang: "sh", block: true)
#code("lit")
#f(lines)
#terminal("defined in good.typ")
#let k = (x) => f(x)
#k("lit")
#raw(```sh
mt ls
```.text, block: true)
EOF

cat >"$tmp/good.typ" <<'EOF'
= Good
#raw(read("x.json"), lang: "json", block: true)
#raw(render-slices(slices, (("id", "note"),), comment: "#"), lang: "toml", block: true)
#raw(fact.code, block: true)
#raw(dsl.join("\n"), lang: "typ", block: true)
#raw("inline") and #raw("x", block: false)
inline ```sh mt ls``` and `code`
// ```sh
// mt ls
// ```
/// ```typst
/// #x
/// ```
#let terminal(body) = raw(body.trim("\n"), block: true)
#terminal(read("out.txt"))
#let pick(fact, name) = { let f = fact.items.find(i => i.name == name); raw(f.code, block: true) }
#pick(fw, "ice")
#let pick2(fact, name) = raw(fact.at(name), block: true)
#pick2(fw, "ice")
#let slug = "ls"
#terminal(read("../generated/" + slug + ".txt"))
#let capture(name) = terminal(read("../generated/" + name + ".txt"))
#capture("ls")
#let pick3(fact, name) = raw(first-of(fact, i => i.name == name), block: true)
#pick3(fw, "ice")
#let fmt = (x) => "- " + x
#raw(fmt(item), block: true)
#let s = "a ``` b"
EOF

cat >"$tmp/markers.typ" <<'EOF'
// docs:hand-typed a grammar sketch no fixture can hold
```
expr := term
```
// docs:hand-typed
```
x
```
// docs:hand-typed nothing below this one

```
y
```
EOF

(cd "$tmp" && doc_blocks_scan bad.typ good.typ markers.typ) | cut -f1-3 >"$tmp/got" ||
    fail_control "the scan itself failed"
{
    for l in 2 5 8 11 12 17 18 20 22 24 26 28 29 30 32 33; do printf 'bad.typ\t%s\tblock\n' "$l"; done
    printf 'markers.typ\t%s\t%s\n' 5 marker 6 block 9 marker 11 block
} >"$tmp/want"
diff "$tmp/want" "$tmp/got" >"$tmp/diff" ||
    fail_control "scan rows differ from the fixture's (- want, + got): $(cat "$tmp/diff")"

# The list rule: an unlisted file with a block and a listed file without one
# both fail by name; listing exactly the files with blocks passes.
printf 'a.typ\t3\tblock\ta fenced block\n' >"$tmp/scan"
rc=0
doc_blocks_judge "$tmp/scan" b.typ 2>"$tmp/judge" || rc=$?
if [ "$rc" -eq 0 ] || ! grep -qF 'a.typ:3: a fenced block is typed by hand' "$tmp/judge" ||
    ! grep -qF 'b.typ has no hand-typed block left' "$tmp/judge"; then
    fail_control "the list rule passed or misnamed: $(cat "$tmp/judge")"
fi
doc_blocks_judge "$tmp/scan" a.typ 2>"$tmp/judge" ||
    fail_control "the list rule refused a listed file with a block: $(cat "$tmp/judge")"
