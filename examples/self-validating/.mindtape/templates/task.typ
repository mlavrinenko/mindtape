#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot(
    "title",
    prompt: "What changes",
    example: "Export the report as CSV",
  ),
  x-goals: slot(
    "goals",
    kind: "raw",
    prompt: "One verdict per goal in goals.typ: (fast-onboarding: \"...\", ...)",
    example: ```typc
    (
      fast-onboarding: "a new user exports without reading the docs",
      no-lock-in: "CSV opens in every spreadsheet tool",
    )
    ```,
  ),
)
