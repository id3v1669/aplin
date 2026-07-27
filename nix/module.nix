self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.programs.aplin;
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

    settings = mkOption {
      description = "KDL config for `aplin`";
      default = "";
      type = types.lines;
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
          aplincfg = pkgs.writeText "config.kdl" cfg.settings;
          aplinCmd =
            if cfg.settings != ""
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
