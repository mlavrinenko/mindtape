#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Put the changelog section in the GitHub release body",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the release page says what changed],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 14),
)

== Problem

+ Pre-release review for 0.3.0, reading how `release.yml` makes the GitHub release
+ 🔴 #raw(".github/workflows/release.yml"): The release step builds its body from `generate_release_notes`, which lists merged PRs. This repo lands by fast-forward with no PRs, so the v0.3.0 page would show only a compare link, not the `CHANGELOG.md` section

```
- name: Create GitHub Release
  uses: softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65 # v2.6.2
  with:
    tag_name: ${{ needs.version-check.outputs.tag }}
    make_latest: ${{ needs.version-check.outputs.latest }}
    generate_release_notes: true
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".github/workflows/release.yml").

== TO-BE

- 🟢 The v0.3.0 release page shows the `## [0.3.0]` section of `CHANGELOG.md` as its body.


== Changes

- `.github/workflows/release.yml`: extract the tag's section from `CHANGELOG.md` into a file and pass it as `body_path`, in place of `generate_release_notes`.


== Rollback

- Revert the `release.yml` change.


== Out of scope

- What the changelog says. Its own finding covers that.


== Dogfood

Every release of this repo carries its changelog section on GitHub.
