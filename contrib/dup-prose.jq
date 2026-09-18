# dup-prose.jq — report near-duplicate PROSE sentences across the docs.
#
# Reads a JSON array of {file, text} objects on stdin, one per doc SOURCE
# (never a generated render — a README and its docs/src source would otherwise
# differ only by the render, and every pair would be a false positive). Run it
# through `just dup-prose`; it is advisory, not a gate: the threshold is a
# judgement call, and this report is an input to a human deciding where a fact
# should live, not a verdict.
#
# The pipeline, per file: strip Typst/Markdown noise, split into sentences,
# keep the ones long enough to be prose, reduce each to the set of its
# 4-word shingles, index shingle -> sentence, tally the shingles each cross-file
# pair shares, and print every pair whose Jaccard overlap clears $min. A
# one-word edit changes 4 of a ~12-shingle sentence, so 0.45 catches the
# copy-paste-with-a-tweak case that an exact-match tool misses.
#
# Why jq and not a script in another language: jq is already the repo's
# text-munging tool (www/*.jq, contrib/jq/) and the whole job is set arithmetic
# over word shingles, so this audit adds no runtime to the devshell.

# --- knobs -------------------------------------------------------------------

def min_words: 7; # shorter strings are headings, labels and markup fragments
def shingle_k: 4;
def threshold: ($ARGS.named.min // 0.45);

# --- noise removal -----------------------------------------------------------

def prose:
  gsub("(?s)```.*?```"; " ")                    # fenced code blocks
  | gsub("`[^`]*`"; " ")                        # inline code spans
  | gsub("(?m)^\\s*//.*$"; " ")                 # Typst/Markdown line comments
  | gsub("(?m)^\\s*#.*$"; " ")                  # whole Typst directive/code lines
  | gsub("(?m)^\\s*(route|label|title|summary|home):.*$"; " ") # page metadata
  | gsub("#[a-zA-Z][a-zA-Z0-9_.-]*\\([^()]*\\)"; " ")          # inline calls
  | gsub("#[a-zA-Z][a-zA-Z0-9_.-]*"; " ")       # call/bracket markers
  | gsub("\\\\[#@]"; " ")                       # escaped markup characters
  | gsub("[<>\\[\\]{}*_`|]"; " ");              # markup punctuation

def sentences:
  gsub("(?<=[.!?:;])\\s+"; "\n")
  | split("\n")[]
  | gsub("\\s+"; " ") | ltrimstr(" ") | rtrimstr(" ")
  | select(length > 0);

def words: [scan("[a-zA-Z0-9']+") | ascii_downcase];

def units($f):
  prose
  | sentences as $s
  | $s | words as $w
  | select(($w | length) >= min_words)
  | {
      file: $f,
      s: $s,
      sh: ([range(0; (($w | length) - shingle_k + 1)) as $i
            | $w[$i:$i + shingle_k] | join(" ")] | unique),
    }
  | select((.sh | length) > 0);

# --- report ------------------------------------------------------------------

[.[] as $doc | $doc.text | units($doc.file)] as $units
| (reduce $units[] as $u ({};
      reduce $u.sh[] as $sh (.; .[$sh] = ((.[$sh] // []) + [$u])))
  ) as $index
| [
    $index | to_entries[] | .value as $bucket
    | select(($bucket | length) > 1)
    | range(0; ($bucket | length)) as $i
    | range($i + 1; ($bucket | length)) as $j
    | {a: $bucket[$i], b: $bucket[$j]}
  ]
| reduce .[] as $p ({};
    ($p.a.file + "#" + ($p.a.s) + "|" + $p.b.file + "#" + ($p.b.s)) as $k
    | .[$k] = ((.[$k] // ($p + {shared: 0})) | .shared += 1))
| [
    .[] | select(.a.file != .b.file)
    | . + {jaccard: (.shared / ((.a.sh | length) + (.b.sh | length) - .shared))}
    | select(.jaccard >= threshold)
  ]
| sort_by(-.jaccard)
| (.[] | "\(.jaccard * 100 | round / 100)\t\(.a.file) ~ \(.b.file)\n  A: \(.a.s)\n  B: \(.b.s)\n"),
  "\($units | length) prose sentences -> \(length) cross-file near-duplicate pair(s)"
