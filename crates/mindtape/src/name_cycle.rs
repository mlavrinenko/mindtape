//! Cycle detection over a name-reference graph, shared by the config sections
//! that build one.
//!
//! Two do today: `[views.<name>]`, whose filter may reference another view
//! (`in_view("other")` / `@other`), and `[dicts.<name>]`, whose `include` names
//! other dicts. Both refuse a cycle at config load and both name the cycle in
//! the message, so the walk lives here once rather than as a copy per section —
//! a second copy would be a duplicated block the copy-paste gate refuses, and a
//! fix applied to one of them only.

use std::collections::BTreeMap;

/// State marks for the cycle-detection DFS.
const UNSEEN: u8 = 0;
const ACTIVE: u8 = 1;
const DONE: u8 = 2;

/// Depth-first search for a cycle in a name-reference graph. Returns the names
/// on the first cycle found (closing back to the first), or `None` if acyclic.
/// Edges to a target absent from `refs` (an unknown reference, reported
/// separately by each caller) are ignored — they cannot join a cycle.
pub(crate) fn find_cycle(refs: &BTreeMap<&str, Vec<String>>) -> Option<Vec<String>> {
    let mut state: BTreeMap<String, u8> = BTreeMap::new();
    for start in refs.keys() {
        let mut stack: Vec<String> = Vec::new();
        if let Some(cycle) = dfs(start, refs, &mut state, &mut stack) {
            return Some(cycle);
        }
    }
    None
}

fn dfs(
    node: &str,
    refs: &BTreeMap<&str, Vec<String>>,
    state: &mut BTreeMap<String, u8>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    match state.get(node).copied().unwrap_or(UNSEEN) {
        DONE => return None,
        ACTIVE => {
            let mut cycle: Vec<String> = stack.iter().skip_while(|n| *n != node).cloned().collect();
            cycle.push(node.to_owned());
            return Some(cycle);
        }
        _ => {}
    }
    state.insert(node.to_owned(), ACTIVE);
    stack.push(node.to_owned());
    if let Some(targets) = refs.get(node) {
        for target in targets {
            if refs.contains_key(target.as_str())
                && let Some(cycle) = dfs(target, refs, state, stack)
            {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    state.insert(node.to_owned(), DONE);
    None
}
