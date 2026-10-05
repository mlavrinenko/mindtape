# .just/scripts/check-doc-spans.jq — the checker .just/scripts/check-doc-spans.sh
# runs over every inline code span it extracts. Input is one raw line per span,
# `<file>\t<line>\t<span>`; output is one finding per line, nothing when clean.
#
# It judges config keys only. `mt` command lines go to `mt _cli --check-argv`
# in check-doc-spans.sh. Each argument arrives through --slurpfile, so each is
# a one-element array: $schema is schema/config.schema.json and $files lists
# tracked file basenames.
$schema[0] as $schema | $files[0] as $files |

def lev($a; $b):
  ($a | explode) as $s | ($b | explode) as $t
  | reduce range(0; $s | length) as $i ([range(0; ($t | length) + 1)];
      . as $prev
      | reduce range(0; $t | length) as $j ([$i + 1];
          . + [[$prev[$j + 1] + 1, .[$j] + 1,
                $prev[$j] + (if $s[$i] == $t[$j] then 0 else 1 end)] | min]))
  | .[-1];
def closest($name; $cands):
  if ($cands | length) == 0 then "none"
  else $cands | map({c: ., d: lev($name; .)}) | min_by(.d) | .c end;

# A word the doc writes as a stand-in: `<task>`, `TERM…`, `N`, `*`.
def placeholder: test("[<>…*]|\\.\\.\\.|^[A-Z][A-Z_]*$");

# ---- config keys -----------------------------------------------------------

def deref:
  if has("$ref") then $schema["$defs"][.["$ref"] | ltrimstr("#/$defs/")] | deref
  else . end;
def alts: deref | if has("anyOf") then .anyOf[] | alts elif has("oneOf") then .oneOf[] | alts else . end;
def types: .type | if type == "array" then .[] else values end;
def is_obj: has("properties") or any(types; . == "object");
def is_arr: any(types; . == "array");
# "aot" is an array of tables, written `[[...]]`; "any" sits below a
# placeholder segment, where every key and form is accepted.
def kind:
  if .any then "any" else
    [alts] as $a
    | if any($a[]; is_arr and any(.items | alts; is_obj)) then "aot"
      elif any($a[]; is_obj) then "table"
      else "value" end
  end;

# One path segment down. An array is entered through its items, as a TOML
# header after `[[x]]` enters the last element. A map (additionalProperties)
# takes any key.
def step($seg):
  if .any then {ok: .} else
    [alts | if is_arr then .items | alts else . end | select(is_obj)] as $o
    | (first($o[] | .additionalProperties | select(type == "object")) // null) as $map
    | if $seg | placeholder then {ok: ($map // {any: true})}
      else (first($o[] | .properties[$seg] | values) // $map) as $n
        | if $n then {ok: $n} else {miss: [$o[] | .properties // {} | keys[]] | unique} end
      end
  end;

# Resolves $segs from $node; `.err` names the first segment that is no key.
def resolve($node; $segs; $base):
  reduce range(0; $segs | length) as $i ({node: $node, err: null};
    if .err then . else
      (.node | step($segs[$i])) as $r
      | if $r.ok then .node = $r.ok
        else .err = "\(($base + $segs[:$i]) | join(".") | if . == "" then "the top level" else . end) has no key \($segs[$i]); closest: \(closest($segs[$i]; $r.miss))"
        end
    end);

def seg_ok: test("^(?:[A-Za-z0-9_-]+|<[^>]+>|\\*)$");
def rooted: .[0] as $root | ($schema.properties | has($root)) and all(.[]; seg_ok);

def header_findings:
  capture("^(?<open>\\[\\[?)(?<path>[^\\[\\]\\s]+)(?<close>\\]\\]?)(?:\\s+(?<rest>.*))?$")
  | select((.open | length) == (.close | length))
  | (.path | split(".")) as $segs
  | select($segs | rooted)
  | (if .open == "[" then "[\(.path)]" else "[[\(.path)]]" end) as $written
  | resolve($schema; $segs; []) as $r
  | if $r.err then $r.err
    else ($r.node | kind) as $k
      | if .open == "[" and $k == "aot" then "\($written) names an array of tables; write [[\(.path)]]"
        elif .open == "[[" and $k == "table" then "\($written) names a table; write [\(.path)]"
        elif $k == "value" then "\($written) names a value, not a table"
        else
          (.rest // "" | capture("^(?<key>[A-Za-z0-9_.-]+)\\s*(?:=.*)?$") | .key | split(".")) as $keys
          | resolve($r.node; $keys; $segs) | .err | values
        end
    end;

def keypath_findings:
  select(test("^[a-z][a-z0-9_-]*(?:\\.(?:[a-z0-9_-]+|<[^>]+>|\\*))+(?:\\s*=.*)?$"))
  | sub("\\s*=.*$"; "") as $path
  | select(($files | index($path)) | not)
  | ($path | split(".")) as $segs
  | select($segs | rooted)
  | resolve($schema; $segs; []) | .err | values;

split("\t") as [$file, $line, $span]
| ($span | gsub("^\\s+|\\s+$"; "")) as $s
| ($s | if startswith("[") then header_findings
        else keypath_findings end)
| "check-doc-spans: \($file):\($line): `\($s)`: \(.)"
