//! `mt views` — list or inspect named views (`[views]`).

use std::collections::BTreeMap;

use crate::diag::Result;
use mindtape::ViewSpec;

use crate::util;

/// Arguments for `views`.
#[derive(Debug, clap::Args)]
pub struct ViewsArgs {
    /// Show this view's full definition instead of the list.
    pub name: Option<String>,

    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// Run the `views` command.
///
/// # Errors
/// Returns an error if the project cannot be opened or a named view is unknown.
pub fn run(args: &ViewsArgs) -> Result<()> {
    let project = util::open()?;
    let views = &project.config.views;

    if let Some(name) = &args.name {
        let spec = views
            .get(name)
            .ok_or_else(|| util::unknown_view_error(name, views))?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(spec)?);
        } else {
            print_detail(name, spec);
        }
        return Ok(());
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(views)?);
    } else if views.is_empty() {
        println!("no views defined (add a [views.<name>] table to .mindtape/config.toml)");
    } else {
        print_list(views);
    }
    Ok(())
}

/// The filter as shown to a human; empty means match-all.
fn filter_display(spec: &ViewSpec) -> &str {
    if spec.filter.trim().is_empty() {
        "(all)"
    } else {
        &spec.filter
    }
}

/// A ` [sort, columns]` tail naming the extra facets a view sets, or empty.
fn facet_tail(spec: &ViewSpec) -> String {
    let mut facets = Vec::new();
    if !spec.sort.is_empty() {
        facets.push("sort");
    }
    if !spec.columns.is_empty() {
        facets.push("columns");
    }
    if facets.is_empty() {
        String::new()
    } else {
        format!("  [{}]", facets.join(", "))
    }
}

fn print_list(views: &BTreeMap<String, ViewSpec>) {
    let width = views.keys().map(String::len).max().unwrap_or(0);
    for (name, spec) in views {
        println!(
            "{name:<width$}  {}{}",
            filter_display(spec),
            facet_tail(spec)
        );
    }
}

fn print_detail(name: &str, spec: &ViewSpec) {
    println!("view:   {name}");
    if let Some(description) = &spec.description {
        println!("about:  {description}");
    }
    println!("filter: {}", filter_display(spec));
    if !spec.sort.is_empty() {
        println!("sort:   {}", spec.sort.join(", "));
    }
    if !spec.columns.is_empty() {
        println!("cols:   {}", spec.columns.join(", "));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use mindtape::parse_config;

    use super::{facet_tail, filter_display, print_detail, print_list};

    fn views() -> std::collections::BTreeMap<String, mindtape::ViewSpec> {
        parse_config(
            "[views.hot]\nfilter = 'priority > 5'\nsort = ['priority:desc']\n\
             columns = ['status', 'title']\ndescription = 'Hot stuff'\n\
             [views.all]\nfilter = ''\n",
        )
        .unwrap()
        .views
    }

    #[test]
    fn filter_display_marks_empty_as_all() {
        let v = views();
        assert_eq!(filter_display(v.get("all").unwrap()), "(all)");
        assert_eq!(filter_display(v.get("hot").unwrap()), "priority > 5");
    }

    #[test]
    fn facet_tail_lists_present_facets() {
        let v = views();
        assert_eq!(facet_tail(v.get("hot").unwrap()), "  [sort, columns]");
        assert_eq!(facet_tail(v.get("all").unwrap()), "");
    }

    #[test]
    fn printers_do_not_panic() {
        let v = views();
        print_list(&v);
        print_detail("hot", v.get("hot").unwrap());
    }
}
