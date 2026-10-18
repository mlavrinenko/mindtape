#!/usr/bin/env bash
# Body of `just mutants status [DIR]`. cargo-mutants prints no ETA, so derive
# one from the two files a run writes as it goes: mutants.json (the fixed list
# to test) and outcomes.json (one entry per finished mutant, after the opening
# Baseline entry). Needs jq >= 1.7 for `localtime`.
set -eo pipefail
dir="${1:-mutants.out}"
if [ ! -f "$dir/outcomes.json" ]; then
    echo "no $dir/outcomes.json, so no sweep has written there"
    exit 1
fi
# A finished run's elapsed is its own wall time, not the clock since it
# started, or a `done` line reports hours the run did not take.
jq -nr \
  --slurpfile o "$dir/outcomes.json" \
  --slurpfile m "$dir/mutants.json" '
  def epoch: sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601;
  ($m[0] | length) as $total
| ($o[0].outcomes | length - 1) as $done
| (if $o[0].end_time != null then ($o[0].end_time | epoch) else now end) as $end
| ($end - ($o[0].start_time | epoch)) as $el
| (if $done > 0 and $el > 0 then $done / $el else 0 end) as $rate
| (if $rate > 0 then ($total - $done) / $rate else null end) as $eta
| "\($done)/\($total)  \($el/60|floor)min elapsed  \((($rate*60)*100|round)/100)/min  "
  + (if $o[0].end_time != null or $eta == null or $eta <= 0 then "done"
     else "ETA \($eta/60|floor)min -> \((now + $eta) | localtime | strftime("%H:%M"))"
     end)
'
