#![allow(clippy::unwrap_used)]

use super::annotate;
use crate::config::Config;
use crate::link::Link;
use crate::model::Task;
use crate::test_support::{dep_config, dep_edge, dep_task as task, link_edge};

fn config() -> Config {
    dep_config("")
}

/// The same legend plus a custom kind flagged blocking, so a project opting an
/// extra kind into readiness gets it in the graph ranks too.
fn config_with_blocking_kind() -> Config {
    dep_config("[[tasks.links.kinds]]\nname = \"waits-for\"\nblocking = true\n")
}

/// `(topo, blocks)` for the task with `id`, after ranking `tasks` under
/// `config`.
fn rank(tasks: &mut [Task], config: &Config, id: &str) -> (u32, u32) {
    annotate(tasks, config);
    let found = tasks
        .iter()
        .find(|task| task.id == id)
        .expect("fixture must contain the id");
    (found.topo, found.blocks)
}

/// A two-task fixture: an open `a` carrying exactly `links`, and a plain open
/// `b` for those links to point at.
fn pair(links: Vec<Link>) -> [Task; 2] {
    let mut a = task("a", "wip", &[]);
    a.links = links;
    [a, task("b", "wip", &[])]
}

#[test]
fn an_isolated_task_ranks_zero_on_both() {
    let mut tasks = [task("a", "wip", &[])];
    assert_eq!(rank(&mut tasks, &config(), "a"), (0, 0));
}

#[test]
fn topo_is_one_more_than_the_deepest_dependency() {
    // c <- b <- a: `a` depends on `b`, which depends on `c`.
    let mut tasks = [
        task("a", "wip", &["b.typ"]),
        task("b", "wip", &["c.typ"]),
        task("c", "wip", &[]),
    ];
    let config = config();
    annotate(&mut tasks, &config);
    let topo: Vec<u32> = tasks.iter().map(|task| task.topo).collect();
    assert_eq!(topo, vec![2, 1, 0]);
}

#[test]
fn topo_takes_the_longest_path_through_a_diamond() {
    // `top` depends on both `left` and `right`; `left` also depends on `deep`.
    // The long arm decides the level, so `top` is 2 above `deep`, not 1.
    let mut tasks = [
        task("deep", "wip", &[]),
        task("left", "wip", &["deep.typ"]),
        task("right", "wip", &[]),
        task("top", "wip", &["left.typ", "right.typ"]),
    ];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "top").0, 2);
    assert_eq!(rank(&mut tasks, &config, "left").0, 1);
    assert_eq!(rank(&mut tasks, &config, "right").0, 0);
}

#[test]
fn independent_tasks_share_a_topo_level() {
    // The property that makes `topo` sortable with a tiebreak: it is a level,
    // not a linear position, so two unrelated tasks tie and a following
    // `--sort` spec decides between them.
    let mut tasks = [task("a", "wip", &[]), task("b", "wip", &[])];
    let config = config();
    annotate(&mut tasks, &config);
    assert_eq!(tasks[0].topo, tasks[1].topo);
}

#[test]
fn topo_is_status_blind() {
    // A done dependency still sits before its dependent in the graph, so it
    // still raises the dependent's level. Only `blocks` reads status.
    let mut tasks = [task("a", "wip", &["b.typ"]), task("b", "done", &[])];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "a").0, 1);
}

#[test]
fn blocks_counts_transitive_dependents() {
    // c <- b <- a: finishing `c` eventually unblocks both `b` and `a`.
    let mut tasks = [
        task("a", "wip", &["b.typ"]),
        task("b", "wip", &["c.typ"]),
        task("c", "wip", &[]),
    ];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "c").1, 2);
    assert_eq!(rank(&mut tasks, &config, "b").1, 1);
    assert_eq!(rank(&mut tasks, &config, "a").1, 0);
}

#[test]
fn blocks_counts_a_shared_dependent_once() {
    // `top` depends on `left` and `right`, both depending on `deep`. `deep`
    // blocks three distinct tasks, not four — the walk is over a set.
    let mut tasks = [
        task("deep", "wip", &[]),
        task("left", "wip", &["deep.typ"]),
        task("right", "wip", &["deep.typ"]),
        task("top", "wip", &["left.typ", "right.typ"]),
    ];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "deep").1, 3);
}

#[test]
fn a_terminal_dependent_does_not_count() {
    // `a` is done, so it is not waiting on `b` any more.
    let mut tasks = [task("a", "done", &["b.typ"]), task("b", "wip", &[])];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "b").1, 0);
}

#[test]
fn blocks_does_not_walk_through_a_terminal_dependent() {
    // c <- b <- a with `b` done: `a` waits on `b`, and `b` is already
    // satisfied, so `c` unblocks nothing at all.
    let mut tasks = [
        task("a", "wip", &["b.typ"]),
        task("b", "done", &["c.typ"]),
        task("c", "wip", &[]),
    ];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "c").1, 0);
}

#[test]
fn a_terminal_task_blocks_nothing() {
    let mut tasks = [task("a", "wip", &["b.typ"]), task("b", "done", &[])];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "b").1, 0);
}

#[test]
fn blocks_shrinks_as_a_dependent_closes() {
    // The declared consequence of counting only non-terminal dependents:
    // leverage is a live measure, so closing `a` drops `b`'s count.
    let config = config();
    let mut open = [task("a", "wip", &["b.typ"]), task("b", "wip", &[])];
    assert_eq!(rank(&mut open, &config, "b").1, 1);
    let mut closed = [task("a", "done", &["b.typ"]), task("b", "wip", &[])];
    assert_eq!(rank(&mut closed, &config, "b").1, 0);
}

#[test]
fn a_cycle_ranks_without_hanging() {
    // `mt check` rejects this, but a sort must survive an unchecked tree.
    let mut tasks = [
        task("a", "wip", &["b.typ"]),
        task("b", "wip", &["c.typ"]),
        task("c", "wip", &["a.typ"]),
    ];
    let config = config();
    annotate(&mut tasks, &config);
    // Every member reaches the other two around the loop, and itself is never
    // counted.
    for task in &tasks {
        assert_eq!(task.blocks, 2, "{} blocks", task.id);
        assert_eq!(task.topo, 0, "{} topo", task.id);
    }
}

#[test]
fn a_task_hanging_off_a_cycle_still_ranks() {
    let mut tasks = [
        task("a", "wip", &["b.typ"]),
        task("b", "wip", &["a.typ"]),
        task("tail", "wip", &["a.typ"]),
    ];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "tail").1, 0);
    assert_eq!(rank(&mut tasks, &config, "a").1, 2);
}

#[test]
fn a_self_edge_orders_nothing() {
    let mut tasks = [task("a", "wip", &["a.typ"])];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "a"), (0, 0));
}

#[test]
fn a_dangling_target_is_not_an_edge() {
    let mut tasks = [task("a", "wip", &["missing.typ"])];
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "a"), (0, 0));
}

#[test]
fn a_non_blocking_kind_is_not_an_edge() {
    let mut tasks = pair(vec![link_edge("b.typ", "supersedes")]);
    let config = config();
    assert_eq!(rank(&mut tasks, &config, "a"), (0, 0));
    assert_eq!(rank(&mut tasks, &config, "b"), (0, 0));
}

#[test]
fn a_custom_blocking_kind_is_an_edge() {
    // `ready`/`blocked` honour a project's `blocking = true` kind, and so do
    // the graph ranks — one edge set behind all four.
    let mut tasks = pair(vec![link_edge("b.typ", "waits-for")]);
    let config = config_with_blocking_kind();
    assert_eq!(rank(&mut tasks, &config, "a"), (1, 0));
    assert_eq!(rank(&mut tasks, &config, "b"), (0, 1));
}

#[test]
fn a_pair_linked_twice_counts_as_one_edge() {
    // Two blocking kinds between the same pair must not double the dependency
    // count Kahn's algorithm counts down, or `a` would never reach level 1.
    let mut tasks = pair(vec![dep_edge("b.typ"), link_edge("b.typ", "waits-for")]);
    let config = config_with_blocking_kind();
    assert_eq!(rank(&mut tasks, &config, "a"), (1, 0));
    assert_eq!(rank(&mut tasks, &config, "b"), (0, 1));
}

#[test]
fn ranking_is_written_onto_every_task() {
    // `annotate` zips its computed ranks onto the slice positionally; this
    // pins that a task never receives its neighbour's numbers.
    let mut tasks = [
        task("a", "wip", &["b.typ"]),
        task("b", "wip", &[]),
        task("c", "wip", &[]),
    ];
    let config = config();
    annotate(&mut tasks, &config);
    let pairs: Vec<(&str, u32, u32)> = tasks
        .iter()
        .map(|task| (task.id.as_str(), task.topo, task.blocks))
        .collect();
    assert_eq!(pairs, vec![("a", 1, 0), ("b", 0, 1), ("c", 0, 0)]);
}
