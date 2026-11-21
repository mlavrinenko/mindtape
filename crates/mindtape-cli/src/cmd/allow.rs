//! `mt allow`: let the commands the project's config runs, its
//! `[[tasks.waits]]` checkers, `[[on.flip]]` and `[[on.rm]]` `run` actions and
//! `[go]` and `[add]` editors, run in this repository. See
//! [`mindtape::allow`].

use serde::Serialize;

use crate::cmd::schema::ALLOW_STATUS;
use crate::diag::Result;
use crate::render::envelope::Envelope;
use crate::util;
use mindtape::allow::{Site, SiteKind};
use mindtape::shell::ConfigCommand;

/// Arguments for `allow`.
#[derive(Debug, clap::Args)]
pub struct AllowArgs {
    /// List the commands without recording anything.
    ///
    /// It prints whether this repository allows them all, then one row per
    /// command.
    #[arg(long)]
    pub status: bool,

    /// Print the list as JSON. Implies `--status`.
    #[arg(long)]
    pub json: bool,
}

/// What `mt allow --status --json` prints.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AllowStatusPayload {
    /// Whether the user allowed every command the config declares for this
    /// repository.
    pub allowed: bool,
    /// Every command the config declares, in declaration order.
    pub sites: Vec<AllowSiteJson>,
}

/// One site in the `allow-status` output.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AllowSiteJson {
    /// Whether the site gates a write or opens an editor.
    pub kind: SiteKind,
    /// The config section that declares it: `waits`, `on.flip`, `on.rm`,
    /// `go` or `add`.
    pub moment: String,
    /// The hook's `name`, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The command as its config spells it: a shell line, or a program and its
    /// arguments.
    pub command: CommandJson,
}

/// A config command in its JSON shape.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum CommandJson {
    /// A command line for the platform shell.
    Line(String),
    /// A program and its arguments, run without a shell.
    Argv(Vec<String>),
}

/// Run the `allow` command: print every command the config runs, then record
/// them as allowed for this repository and each of its worktrees.
///
/// With `--status`, or with `--json`, it prints the same list and doesn't
/// record anything, so a reader can inspect the commands before trusting them.
///
/// It reads the config only. Loading the tasks would run the checkers it is
/// about to allow.
///
/// # Errors
/// Returns an error if the project or its config does not read, or the write of
/// the approval fails.
pub fn run(args: &AllowArgs) -> Result<()> {
    let root = util::resolve_root()?;
    let config = mindtape::load_config(&root)?;
    let sites = mindtape::allow::sites(&config);
    if sites.is_empty() {
        if args.json {
            return print_json(true, &sites);
        }
        println!("the config runs no commands, so there is nothing to allow");
        return Ok(());
    }
    let commands = mindtape::allow::commands(&config);
    if args.status || args.json {
        let allowed = mindtape::allow::is_allowed(&root, &commands);
        if args.json {
            return print_json(allowed, &sites);
        }
        print_table(allowed, &sites);
        return Ok(());
    }
    mindtape::allow::record(&root, &commands)?;
    print_table(true, &sites);
    Ok(())
}

/// Print the sites under one status line for the set: `allowed` or
/// `not allowed`, then one row per site with its moment and command.
fn print_table(allowed: bool, sites: &[Site]) {
    let status = if allowed { "allowed" } else { "not allowed" };
    println!("{status} in this repository and its worktrees:");
    let width = sites
        .iter()
        .map(|site| site.moment.len())
        .max()
        .unwrap_or(0);
    for site in sites {
        println!("  {:width$}  {}", site.moment, site.command);
    }
}

/// Print the list as the `allow-status` envelope.
fn print_json(allowed: bool, sites: &[Site]) -> Result<()> {
    let payload = AllowStatusPayload {
        allowed,
        sites: sites.iter().map(AllowSiteJson::of).collect(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&Envelope::new(ALLOW_STATUS, payload))?
    );
    Ok(())
}

impl AllowSiteJson {
    /// One site as its JSON shape.
    fn of(site: &Site) -> Self {
        Self {
            kind: site.kind,
            moment: site.moment.to_owned(),
            label: site.label.clone(),
            command: CommandJson::of(&site.command),
        }
    }
}

impl CommandJson {
    /// One command as its JSON shape.
    fn of(command: &ConfigCommand) -> Self {
        match command {
            ConfigCommand::Line(line) => Self::Line(line.clone()),
            ConfigCommand::Argv(argv) => Self::Argv(argv.clone()),
        }
    }
}
