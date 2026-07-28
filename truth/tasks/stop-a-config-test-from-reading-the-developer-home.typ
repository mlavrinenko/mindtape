#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Stop a config test from reading the developer home",
  priority: ice(
    impact: 6,
    confidence: 1.0,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (files: 2),
  status: done(
    2026,
    7,
    28,
  )[dropped the duplicate that went through load\_config; the injected-seam twin covers it],
)

== Summary

config::tests::load\_config\_on\_missing\_file\_matches\_parse\_config\_empty asserted that loading config from an empty directory equals parsing an empty config string. load\_config layers in the XDG global defaults at dirs::config\_dir() over mindtape over defaults.toml, which is a supported and documented layer, so the assertion held only on a machine where the developer keeps no such file. The moment one appeared carrying an ls hyperlinks setting, just check went red on a tree with no Rust changes at all. config\_layers\_tests already asserts the same thing through discover\_layers\_with\_xdg with the XDG directory injected, so the failing test contributed no coverage beyond its environment dependence.

== Scope

- Drop the duplicate that goes through load\_config.
- Say in the surviving test why the seam is injected, so the environment-dependent form does not come back.
