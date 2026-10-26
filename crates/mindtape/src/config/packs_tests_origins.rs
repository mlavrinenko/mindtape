//! The keys and parameters a loaded config remembers a pack for, and the
//! file a load warning is placed in. The fixtures write real files, so the
//! origins are the ones `mt config show --origin` prints.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use crate::config::layers::{LayerSource, load_config_layered_from};
use crate::config::locate::ConfigPlace;
use crate::config::pack_registry::{PackOrigin, RuleOrigin};
use crate::config::{Config, KeyOrigin, ParamOrigin};
use crate::test_support::{home_and_project, write_file};

/// A pack that sets the scale, caps the files and declares the cap.
const TEAM: &str = "[params]\nmax-files = 5\n\n[tasks.priority]\nframework = \"ice\"\n\n\
    [[tasks.rules]]\nname = \"small\"\nassert = 'estimate.src-files <= param(\"max-files\")'\n";

/// Load a project whose config is `own`, beside the pack `team.toml`.
fn load(team: &str, own: &str) -> (tempfile::TempDir, PathBuf, Config) {
    let (tmp, home, project) = home_and_project();
    write_file(&project, "team.toml", team);
    write_file(&project, ".mindtape/config.toml", own);
    let config = load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0;
    (tmp, project, config)
}

fn key(config: &Config, name: &str) -> Option<KeyOrigin> {
    config.key_origins().find(|origin| origin.key == name)
}

/// The origin of `tasks.priority.framework` in a project holding `own`.
fn framework_origin(own: &str) -> (RuleOrigin, PathBuf) {
    let (_tmp, project, config) = load(TEAM, own);
    let origin = key(&config, "tasks.priority.framework").unwrap().origin;
    (origin, project)
}

#[test]
fn a_key_names_the_source_that_won_it() {
    let pack = "extends = [\"./team.toml\"]\n";
    let (origin, project) = framework_origin(pack);
    let team = PackOrigin::Path(project.join("team.toml"));
    assert_eq!(origin, RuleOrigin::Pack(team));

    let own = format!("{pack}[tasks.priority]\nframework = \"rice\"\n");
    let (origin, project) = framework_origin(&own);
    let layer = LayerSource::Project(project.join(".mindtape/config.toml"));
    assert_eq!(origin, RuleOrigin::Layer(layer));
}

#[test]
fn a_key_no_pack_sets_has_no_origin() {
    let own = "extends = [\"./team.toml\"]\n[project]\ncode = \"x\"\n";
    let (_tmp, _project, config) = load(TEAM, own);
    assert!(key(&config, "project.code").is_none());
    assert!(key(&config, "params.max-files").is_none());
}

#[test]
fn a_with_value_names_the_layer_that_set_it() {
    let own = "extends = [{ pack = \"./team.toml\", with = { max-files = 8 } }]\n";
    let (_tmp, project, config) = load(TEAM, own);
    let params: Vec<&ParamOrigin> = config.param_origins().collect();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "max-files");
    assert_eq!(params[0].value, "8");
    assert_eq!(params[0].pack, PackOrigin::Path(project.join("team.toml")));
    assert_eq!(
        params[0].set_by,
        Some(LayerSource::Project(project.join(".mindtape/config.toml")))
    );
}

#[test]
fn a_default_value_names_no_layer() {
    let (_tmp, _project, config) = load(TEAM, "extends = [\"./team.toml\"]\n");
    let params: Vec<&ParamOrigin> = config.param_origins().collect();
    assert_eq!(params[0].value, "5");
    assert_eq!(params[0].set_by, None);
}

#[test]
fn a_warning_sits_at_the_pack_file_and_line_of_its_key() {
    let team = "[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n\n[[tasks.rules]]\nname = \"r\"\n\
        assert = 'x-machnie == \"alpha\"'\nseverity = \"warn\"\n";
    let (_tmp, _project, config) = load(team, "extends = [\"./team.toml\"]\n");
    let warnings = config.warnings();
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        config.warning_place(&warnings[0]),
        Some(ConfigPlace {
            file: PathBuf::from("team.toml"),
            line: 6
        })
    );
}
