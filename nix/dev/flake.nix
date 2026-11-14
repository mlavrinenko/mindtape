# The dev shell's own inputs. The root flake's `dev` partition reads them
# through flake-parts, so they live in this lock and never in the root one,
# which every flake that takes Mindtape as an input copies. Nothing here is
# built from this flake: update an input with
# `nix flake update --flake ./nix/dev <input>`.
{
  inputs = {
    # Memoized command runner. `mt flip` into a terminal status gates on
    # `mmz --is-fresh --tag gate`, which asserts every gate-tagged rule in
    # .mmz/config.yaml already passed against this worktree instead of
    # running any of them again.
    mmz.url = "github:mlavrinenko/mmz";
    # A second nixpkgs, read for `tombi` and `worktrunk`. The root flake's pin
    # stays where the tinymist/Typst note in shell.nix wants it; it predates
    # worktrunk's package, and tombi >= 1.5 is the first with per-schema
    # `format.rules` that switch key and array-value sorting off (see
    # .config/tombi.toml).
    nixpkgs-tombi.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # qahq ships `mt` too; this repo builds its own, so the cycle is cut.
    qahq = {
      url = "github:mlavrinenko/qahq";
      inputs.mindtape.follows = "";
    };
    # Typst-native static site generator that builds the docs under www/.
    tola.url = "github:tola-rs/tola-ssg/v0.7.1";
    # Vale rules. `vale-styles.nix` builds the style from these revisions, so
    # the root lock stays free of a prose gate only the dev shell reads.
    vale-ai-tells = {
      url = "github:tbhb/vale-ai-tells/v1.37.0";
      flake = false;
    };
    vale-google = {
      url = "github:vale-cli/Google/v0.7.2";
      flake = false;
    };
    vale-microsoft = {
      url = "github:vale-cli/Microsoft/v0.15.2";
      flake = false;
    };
    vale-write-good = {
      url = "github:vale-cli/write-good/v0.4.1";
      flake = false;
    };
  };

  outputs = _: { };
}
