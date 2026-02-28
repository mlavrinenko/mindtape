# NixOS module for the mindtape file-watching indexer service.
# Usage: services.mindtape.enable = true;
self:

{ config, lib, pkgs, ... }:

let
  cfg = config.services.mindtape;

  tomlFormat = pkgs.formats.toml { };

  configFile = tomlFormat.generate "mindtape.toml" {
    database.path = cfg.databasePath;
    watch = map (entry: {
      inherit (entry) path recursive;
    }) cfg.watchPaths;
  };

  verbosityFlags =
    if cfg.verbosity == 0 then [ ]
    else [ ("-" + lib.concatStrings (lib.genList (_: "v") cfg.verbosity)) ];
in
{
  options.services.mindtape = {

    enable = lib.mkEnableOption "mindtape, a file-based task tracker using Typst";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.default;
      description = "The mindtape package to use.";
    };

    watchPaths = lib.mkOption {
      type = lib.types.listOf (lib.types.submodule {
        options = {
          path = lib.mkOption {
            type = lib.types.str;
            description = "Absolute path to a directory of Typst files to watch.";
            example = "/home/user/notes";
          };
          recursive = lib.mkOption {
            type = lib.types.bool;
            default = true;
            description = "Whether to watch subdirectories recursively.";
          };
        };
      });
      default = [ ];
      description = "List of directories to watch for Typst file changes.";
      example = lib.literalExpression ''
        [
          { path = "/home/user/notes"; recursive = true; }
          { path = "/home/user/projects"; recursive = false; }
        ]
      '';
    };

    databasePath = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/mindtape/index.db";
      description = "Path to the SQLite database file for the task index.";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "mindtape";
      description = "User account under which mindtape runs.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "mindtape";
      description = "Group under which mindtape runs.";
    };

    extraConfigPaths = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = ''
        Additional TOML config files to merge after the generated config.
        Watch entries from all files are combined; the first database path wins.
        Useful for keeping user-specific watch paths outside of NixOS configuration
        (e.g. "~/.config/mindtape.toml").
      '';
      example = lib.literalExpression ''[ "/home/user/.config/mindtape.toml" ]'';
    };

    verbosity = lib.mkOption {
      type = lib.types.ints.between 0 3;
      default = 0;
      description = ''
        Logging verbosity level.
        0 = warn (default), 1 = info (-v), 2 = debug (-vv), 3 = trace (-vvv).
      '';
    };
  };

  config = lib.mkIf cfg.enable {

    users.users.${cfg.user} = lib.mkIf (cfg.user == "mindtape") {
      isSystemUser = true;
      group = cfg.group;
      home = "/var/lib/mindtape";
      description = "MindTape service user";
    };

    users.groups.${cfg.group} = lib.mkIf (cfg.group == "mindtape") { };

    systemd.services.mindtape = {
      description = "MindTape file-based task tracker";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = lib.escapeShellArgs (
          [ "${cfg.package}/bin/mindtape" "watch" "--config" "${configFile}" ]
          ++ lib.concatMap (p: [ "--config" p ]) cfg.extraConfigPaths
          ++ verbosityFlags
        );
        Restart = "on-failure";
        RestartSec = 5;

        User = cfg.user;
        Group = cfg.group;

        StateDirectory = "mindtape";
        StateDirectoryMode = "0750";

        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = "read-only";
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictSUIDSGID = true;

        ReadWritePaths = [
          (builtins.dirOf cfg.databasePath)
        ];

        ReadOnlyPaths = cfg.extraConfigPaths;
      };
    };
  };
}
