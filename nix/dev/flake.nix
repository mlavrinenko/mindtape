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
    qahq.url = "github:mlavrinenko/qahq";
    # Typst-native static site generator that builds the docs under www/.
    tola.url = "github:tola-rs/tola-ssg/v0.7.1";
  };

  outputs = _: { };
}
