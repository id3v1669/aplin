self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.aplin;
  format = pkgs.formats.yaml {};
  inherit (pkgs.stdenv.hostPlatform) system;

  inherit (lib) types;
  inherit (lib.modules) mkIf;
  inherit (lib.options) mkOption mkEnableOption;
in {
  options.programs.aplin = {
    enable = mkEnableOption "Headphones helper";

    package = mkOption {
      description = "The package to use for `aplin`";
      default = self.packages.${system}.default;
      type = types.package;
    };

    # right now useless as did not implement cfg reading functionaluity yet
    settings = mkOption {
      description = "The config to use for `aplin` syntax and samples could found in [repo](https://github.com/id3v1669/aplin).";
      default = {};
      type = format.type;
    };
  };

  config = mkIf cfg.enable {
    systemd.user.services.aplin = {
      Unit = {
        Description = "Headphones helper service";
        PartOf = ["default.target"];
      };
      Service = {
        ExecStart = let
          aplincfg = pkgs.writeText "aplincfg" "${lib.generators.toYAML {} cfg.settings}";
          aplinCmd =
            if cfg.settings != null
            then "--config ${aplincfg}"
            else "";
        in ''
          ${cfg.package}/bin/aplin ${aplinCmd}
        '';
        Restart = "on-failure";
      };
      Install = {WantedBy = ["default.target"];};
    };
  };
}
