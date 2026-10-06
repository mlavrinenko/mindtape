#import "@local/mindtape:0.2.0": *

#show: task.with(
  // docs:slice comma
  title: "Fix the login redirect",
  assignee: "ada",
  // docs:end
  // docs:slice note
  status: wip(2026, 7, 1)[reproduced on staging],
  // docs:end
  // docs:slice dictionary
  estimate: (days: 1, hours: 2),
  // docs:end
  // docs:slice array
  tags: ("backend",),
  // docs:end
  // docs:slice links
  links: depends-on("add-a-session-store.typ")
    + related("log-failed-logins.typ")[same handler],
  // docs:end
)

= Problem

After login, the app sends every user to `/home`, even one who asked for
`/settings`.

+ Open `/settings` while logged out.
+ Log in.
+ The app shows `/home`.

= Fix

Keep the requested path in the session and redirect to it after login.
// docs:slice hash
Ask in \#backend before you change the session cookie.
// docs:end
