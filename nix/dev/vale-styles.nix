# The Vale style the docs, task and commit-message gates read. Upstream rules
# come from the pinned inputs in nix/dev/flake.nix. The build strips each
# upstream `level` so `.config/vale*.ini` owns every level, applies the local
# edits, and overlays the rules and vocabulary under `.config/vale/local/`. The
# dev shell links the output at `target/vale/styles`, the path the configs name.
#
# `dicts` holds the two verb dictionaries `mt` ships. The `Commits.Imperative`
# rule embeds them, so a commit subject and a task title accept the same verbs.
{
  runCommandLocal,
  yq-go,
  vale-ai-tells,
  vale-google,
  vale-microsoft,
  vale-write-good,
  local,
  dicts,
}:
let
  ai = "${vale-ai-tells}/styles/ai-tells";
  google = "${vale-google}/Google";
  microsoft = "${vale-microsoft}/Microsoft";
  writeGood = "${vale-write-good}/write-good";
in
runCommandLocal "mindtape-vale-styles" { nativeBuildInputs = [ yq-go ]; } ''
  ai=${ai}
  mkdir -p "$out/Prose" "$out/Sep" "$out/Commits" "$out/Plain" "$out/config/vocabularies/Prose" "$out/config/scripts"

  for rule in \
    AICompoundPhrases \
    AnthropomorphicCognition \
    AnthropomorphicJustification \
    BareNames \
    ClosingPleasantries \
    ConclusionMarkers \
    ConsequenceClause \
    DoubleHyphen \
    ExplainerHeadings \
    ExplainerLeads \
    FalseExclusivity \
    FillerPhrases \
    FormalRegister \
    FormalTransitions \
    HedgingPhrases \
    JourneyMetaphors \
    LabelAndExplain \
    NegatedObject \
    NegatedSubject \
    OpeningCliches \
    OverusedVocabulary \
    PromotionalPuffery \
    PseudoCleft \
    RestatementMarkers \
    ServesAsDodge \
    ShellNounCopula \
    ShipOveruse \
    StructureAnnouncements \
    SycophancyMarkers \
    UnpackExplore \
    UrgencyInflation \
    VagueAttributions \
  ; do
    sed '/^level: /d' "$ai/$rule.yml" > "$out/Prose/$rule.yml"
  done

  sed '/^level: /d' "${google}/Latin.yml" > "$out/Prose/Latin.yml"
  sed '/^level: /d' "${google}/Headings.yml" > "$out/Prose/HeadingCase.yml"
  # `Vale` names the tool in a heading. `mt check` matches the two task-shape
  # headings literally, so their case is not a typo.
  yq -i '.exceptions += ["Vale", "AS-IS", "TO-BE"]' "$out/Prose/HeadingCase.yml"

  sed '/^level: /d' "${microsoft}/Passive.yml" > "$out/Prose/PassiveVoice.yml"
  sed '/^level: /d' "${microsoft}/Wordiness.yml" > "$out/Prose/Wordiness.yml"
  sed '/^level: /d' "${microsoft}/Semicolon.yml" > "$out/Prose/Semicolon.yml"
  sed '/^level: /d' "${microsoft}/SentenceLength.yml" > "$out/Prose/SentenceLength.yml"
  sed '/^level: /d' "${microsoft}/HeadingPunctuation.yml" > "$out/Prose/HeadingPunctuation.yml"

  sed '/^level: /d' "${writeGood}/Weasel.yml" > "$out/Prose/Weasel.yml"

  # "implement" and "implementation" name a Rust trait implementation here, so
  # they stay out of the formalism list.
  yq -i '.tokens -= ["implement", "implements", "implementing", "implementation"]' \
    "$out/Prose/FormalRegister.yml"

  # Upstream says `NOTE:` and `OO`; the prose gate rejects the shouted forms.
  sed -i 's/^# NOTE:/# Note:/' "$out/Prose/AnthropomorphicJustification.yml"
  sed -i 's/the OO docs register/the object-oriented docs register/' \
    "$out/Prose/NegatedObject.yml"

  cp -r ${local}/Prose/. "$out/Prose/"
  cp -r ${local}/Sep/. "$out/Sep/"
  cp -r ${local}/Commits/. "$out/Commits/"
  cp -r ${local}/Plain/. "$out/Plain/"
  cp -r ${local}/config/. "$out/config/"

  # The imperative rule reads the same verbs as `word(title, 1)`. Tengo has no
  # file reader, so the build writes the list into the script: one quoted entry
  # per non-comment line of either dictionary, in declaration order.
  {
    printf 'verbs := [\n'
    awk '!/^#/ && NF {
        if (seen++) printf ",\n"
        printf "  \"%s\"", $1
    } END { if (seen) printf "\n" }' \
      ${dicts}/verbs.txt ${dicts}/dev-verbs.txt
    printf ']\n'
    cat ${local}/config/scripts/Imperative.body.tengo
  } > "$out/config/scripts/Imperative.tengo"
''
