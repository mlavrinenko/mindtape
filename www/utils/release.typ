// What the release workflow publishes, read from
// www/generated/release-targets.json, which www/generate-release-targets.sh
// derives from .github/workflows/release.yml.

#let release = json("../generated/release-targets.json")

// The CLI is the one published crate with a `[[bin]]`. The build fails when
// the workflow names none or several, so no page can show a stale crate name.
#let _cli-crates = release.publish.filter(p => p.bin)
#if _cli-crates.len() != 1 {
  panic(
    "release.typ: release-targets.json must name exactly one published crate "
      + "with a [[bin]] (the CLI); found "
      + str(_cli-crates.len()),
  )
}
#let cli-crate = _cli-crates.at(0).crate

// Each target carries the file name the workflow uploads, extension included,
// so the Windows build shows `.exe`.
#let release-targets = release.targets
