// Generation 2: goal exports are append-only; a rename keeps an alias.
// It imports the live goal-calls.typ, so a new goal changes what the check
// demands of an open task. Append-only covers renames, not additions. A task
// that must not move pins generation 1.
#import "/truth/prelude/1/lib.typ": *
#import "/truth/goal-calls.typ": *
