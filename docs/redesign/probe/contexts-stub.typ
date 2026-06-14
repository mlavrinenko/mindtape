// Probe stub: speconaut domain bounded-contexts.
#import "speconaut-stub.typ": bounded-context

#let knowledge-context = bounded-context("knowledge", layer: "functional")
#let adapter-context = bounded-context("adapter", layer: "functional")
#let session-context = bounded-context("session", layer: "functional")
#let specification-context = bounded-context("specification", layer: "functional")
#let codegen-context = bounded-context("codegen", layer: "functional")
