# .just/scripts/check-doc-prose.awk — the scanner check-doc-prose.sh runs over
# its rows. Input is one row per line of text: file, line, kind (prose or
# comment) and text, tab separated. The variables `allowed` (acronyms that may
# be all-caps), `consts` (constants, allowed only as `$NAME`, `${NAME}` or
# `NAME=`), `words` and `phrases` (the history lists) come
# from the script. So do `dash`, a regex of the files whose prose rows must hold
# no em dash, and `dashonly`, a regex of the files that get no other rule.
# Prints "file:line: message" per hit. A line of `-` names a
# schema key in the file column and prints "file: message".
BEGIN {
    n = split(allowed, a, /[ \n]+/)
    for (i = 1; i <= n; i++) ok[a[i]] = 1
    n = split(consts, a, /[ \n]+/)
    for (i = 1; i <= n; i++) const[a[i]] = 1
    nw = split(words, word, / /)
    np = split(phrases, phrase, /\|/)
    for (i = 1; i <= np; i++) {
        re = phrase[i]; gsub(/ /, "[ \t]+", re)
        phrase_re[i] = "(^|[^a-z0-9_])" re "([^a-z0-9_]|$)"
    }
}
# The line with raw spans and quoted text blanked out. raw holds the
# backtick count of the open span; like Typst, a span can wrap lines.
function visible(line,    out, i, n, c, j, run) {
    out = ""; i = 1; n = length(line)
    while (i <= n) {
        c = substr(line, i, 1)
        if (c == "`") {
            j = i; while (j <= n && substr(line, j, 1) == "`") j++
            run = j - i
            if (raw == 0) { if (run != 2) raw = run }
            else if (raw == 1 || run == raw) raw = 0
            out = out " "; i = j; continue
        }
        if (raw) { i++; continue }
        if (c == "\\") { out = out " "; i += 2; continue }
        if (c == "\"") { quoted = !quoted; out = out " "; i++; continue }
        out = out (quoted ? " " : c)
        i++
    }
    return out
}
# An all-caps word of two letters or more: the shape of shouting.
function caps(tok) {
    return tok ~ /^[A-Z][A-Z-]*[A-Z]$/ && tok ~ /[A-Z][A-Z]/
}
# Token k is a shouted word that sits next to token j through spaces only.
function shouted(k) {
    if (k < 1 || k > nt || !caps(tk[k]) || tk[k] in ok || post[k] ~ /^\.[a-z]/) return 0
    return (k > j ? gap[k] : gap[j]) ~ /^ +$/
}
function hit(msg) {
    if (line == "-") printf "%s: %s\n", file, msg
    else printf "%s:%d: %s\n", file, line, msg
}
function count(s, re,    k) {
    k = 0
    while (match(s, re)) { k++; s = substr(s, RSTART + RLENGTH - 1) }
    return k
}
{
    file = $1; line = $2; kind = $3; text = $4
    # An unclosed quote or inline span ends at a blank line, at a gap
    # between rows, or where comment rows meet prose rows. A comment
    # has no fenced blocks, so a stray fence ends there too.
    if (file != pfile || text !~ /[^ \t]/ || kind != pkind \
        || (line != "-" && line != pline + 1)) {
        quoted = 0; tail = ""
        if (raw == 1 || kind == "comment" || file != pfile) raw = 0
    }
    pfile = file; pkind = kind; pline = line
    v = visible(text)
    low = tolower(v)

    # An em dash in rendered text is a hit, one per dash. A comment row is
    # not rendered, so it never counts.
    if (kind == "prose" && dash != "" && file ~ dash) {
        t = v
        k = gsub(/—/, "", t)
        while (k-- > 0) hit("em dash")
    }
    if (dashonly != "" && file ~ dashonly) next

    for (i = 1; i <= nw; i++) {
        k = count(low, "(^|[^a-z0-9_])" word[i] "([^a-z0-9_(]|$)")
        while (k-- > 0) hit("history word \"" word[i] "\"")
    }
    # A phrase can wrap, so the last word of the previous line is
    # joined in front of this one.
    for (i = 1; i <= np; i++) {
        k = count(tail " " low, phrase_re[i])
        while (k-- > 0) hit("history word \"" phrase[i] "\"")
    }
    tail = low
    sub(/[^a-z0-9_]+$/, "", tail)
    sub(/.*[^a-z0-9_]/, "", tail)

    # Tokens with the text before and after each one, so a constant passes only
    # as `$NAME`, `${NAME}` or `NAME=`, and a lone capital beside an all-caps
    # word reads as part of the shouting.
    nt = 0; s = v; off = 0
    while (match(s, /[A-Za-z0-9_$][A-Za-z0-9_-]*/)) {
        nt++; tk[nt] = substr(s, RSTART, RLENGTH)
        st = off + RSTART
        gap[nt] = substr(s, 1, RSTART - 1)
        pre[nt] = substr(v, st > 2 ? st - 2 : 1, st > 2 ? 2 : st - 1)
        post[nt] = substr(v, st + RLENGTH, 2)
        off = st + RLENGTH - 1
        s = substr(s, RSTART + RLENGTH)
    }
    for (j = 1; j <= nt; j++) {
        tok = tk[j]
        if (tok ~ /^[A-Z]$/) {
            if (shouted(j - 1) || shouted(j + 1)) hit("all-caps \"" tok "\"")
            continue
        }
        if (!caps(tok) || tok in ok || post[j] ~ /^\.[a-z]/) continue
        if (tok in const && (post[j] ~ /^=/ || pre[j] ~ /\$\{?$/)) continue
        hit("all-caps \"" tok "\"")
    }
}
