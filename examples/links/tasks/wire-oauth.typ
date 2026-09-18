#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Wire OAuth into the API",
  status: wip(2026, 8, 12)[behind a feature flag],
  priority: 8,
  tags: ("auth", "backend"),
  // docs:slice depends-on-related
  links: depends-on("pick-a-provider.typ")
    + parent("auth-epic.typ")
    + related("write-docs.typ")[the docs ship in the same release],
  // docs:end
)

= Summary

Exchange the provider's code for a token, then map it to a local session.
