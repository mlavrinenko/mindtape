// What this service is for. Every open milestone rates each goal in its
// `x-goals` field, and the `goals` rule in .mindtape/config.toml runs
// `check-goals` on that field.

// docs:slice goals
#let goals = (
  secure: "Only the owner of a session can use it",
  fast: "Every endpoint answers within 100 ms",
)

#let helps(why) = (effect: "helps", why: why)
#let hurts(why) = (effect: "hurts", why: why)
#let neutral = (effect: "neutral")

// Returns one message per problem, and `mt` reports each of them.
#let check-goals(ratings) = {
  if ratings == none {
    return ("rate each goal in x-goals: " + goals.keys().join(", "),)
  }
  let messages = ()
  for key in goals.keys().filter(key => key not in ratings) {
    messages.push("rate the " + key + " goal with helps, hurts or neutral")
  }
  for key in ratings.keys().filter(key => key not in goals) {
    messages.push(key + " is not a goal")
  }
  if ratings.values().all(rating => rating.effect != "helps") {
    messages.push("it helps no goal, so name one it helps or drop it")
  }
  messages
}
// docs:end
