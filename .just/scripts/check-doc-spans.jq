# .just/scripts/check-doc-spans.jq — the checker .just/scripts/check-doc-spans.sh
# runs over every inline code span it extracts. Input is one raw line per span,
# `<file>\t<line>\t<span>`; output is one finding per line, nothing when clean.
#
# Each argument arrives through --slurpfile, so each is a one-element array:
# $cli is `mt _cli --json` with the hidden dump command added, $schema is
# schema/config.schema.json, $statuses lists every status verb the project and
# the examples declare, $files lists tracked file basenames.
$cli[0] as $cli | $schema[0] as $schema | $statuses[0] as $statuses | $files[0] as $files |

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

# ---- mt command spans ------------------------------------------------------

def cmds: $cli.commands | map({key: .name, value: .}) | from_entries;
def flags($specs): [$specs[] | select(.long != null or .short != null)];

# Walks one `mt ...` segment's words. $st carries the command path so far, the
# flags valid at this point, and the subcommand names still expected.
def check_mt($w):
  def err($m): {msg: $m};
  def go($i; $st):
    if $i >= ($w | length) then empty else
    $w[$i] as $t
    | (flags($st.specs) + $cli.globals) as $all
    | if $t == "--" then empty
      elif $st.skip then go($i + 1; $st | .skip = false)
      elif $t | test("^--[A-Za-z]") then
        ($t | ltrimstr("--") | sub("=.*$"; "")) as $n
        | (first($all[] | select(.long == $n)) // null) as $f
        | if $f then go($i + 1; $st | .skip = ($f.kind == "option" and ($t | contains("=") | not)))
          elif $n == "help" or ($n == "version" and $st.path == "mt") then go($i + 1; $st)
          else err("\($st.path) has no flag --\($n); closest: --\(closest($n; [$all[].long | values]))"),
            go($i + 1; $st)
          end
      elif $t | test("^-[A-Za-z]") then
        # A short cluster: each letter is a flag until one takes a value, which
        # is the rest of the word or else the next word.
        ([$t | ltrimstr("-") | explode[] | [.] | implode]) as $cs
        | (reduce range(0; $cs | length) as $k ({bad: [], skip: false, done: false};
            if .done then . else
              (first($all[] | select(.short == $cs[$k])) // null) as $f
              | if $f == null and ($cs[$k] == "h" or ($cs[$k] == "V" and $st.path == "mt")) then .
                elif $f == null then .bad += [$cs[$k]]
                elif $f.kind == "option" then .done = true | .skip = ($k == ($cs | length) - 1)
                else . end
            end)) as $r
        | ($r.bad[] | err("\($st.path) has no flag -\(.); valid: \([$all[].short | values | "-" + .] | sort | join(", "))")),
          go($i + 1; $st | .skip = $r.skip)
      elif $st.path == "mt" then
        if $t | placeholder then empty
        elif cmds[$t] then
          go($i + 1; {path: "mt \($t)", specs: cmds[$t].args, subs: cmds[$t].subcommands, skip: false})
        elif $statuses | index($t) then
          go($i + 1; {path: "mt \($t)", specs: cmds["flip"].args, subs: [], skip: false})
        else err("no subcommand \($t); closest: \(closest($t; (cmds | keys) + $statuses))")
        end
      elif ($st.subs | length) > 0 then
        (first($st.subs[] | select(.name == $t)) // null) as $s
        | if $t | placeholder then empty
          elif $s then go($i + 1; {path: "\($st.path) \($t)", specs: $s.args, subs: [], skip: false})
          else err("\($st.path) has no subcommand \($t); closest: \(closest($t; [$st.subs[].name]))")
          end
      else go($i + 1; $st)
      end
    end;
  # A leading `@name` is worktree sugar that mt rewrites to `-w <path>` before
  # parsing, so the subcommand is the word after it.
  go(if ($w[1] | startswith("@")) then 2 else 1 end;
    {path: "mt", specs: [], subs: [], skip: false});

def mt_findings:
  [splits("\\s+(?:\\||&&|;)\\s+")]
  | .[] | [scan("(?:'[^']*'|\"[^\"]*\"|[^\\s'\"])+")]
  | select(.[0] == "mt" and length > 1)
  | check_mt(.) | .msg;

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
| ($s | if startswith("mt ") or test("\\|\\s+mt\\s") then mt_findings
        elif startswith("[") then header_findings
        else keypath_findings end)
| "check-doc-spans: \($file):\($line): `\($s)`: \(.)"
