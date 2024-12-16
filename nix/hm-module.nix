self:
{ config
, lib
, pkgs
, ...
}:
let
  cfg = config.services.aplin;
  inherit (pkgs.stdenv.hostPlatform) system;

  inherit (lib) types;
  inherit (lib.modules) mkIf;
  inherit (lib.options) mkOption mkEnableOption;
in
{
  options.services.aplin = {
    enable = mkEnableOption "Headphones helper";

    package = mkOption {
      description = "The package to use for `aplin`";
      default = self.packages.${system}.default;
      type = types.package;
    };

    # right now useless as did not implement cfg reading functionaluity yet
    settings = mkOption {
      description = "The config to use for `aplin` syntax and samples could found in [repo](https://github.com/id3v1669/aplin).";
      default = ''
      '';
      type = types.nullOr types.lines;
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.user.services.aplin = {
      description = "headphones helper";
      bindsTo = [ "default.target" ];
      script = let 
        aplincfg = pkgs.writeText "aplincfg" "${cfg.settings}";
        aplinCmd = if cfg.settings != null then "--config ${aplincfg}" else "";
      in ''
        ${cfg.package}/bin/aplin
      '';
      serviceConfig.Restart = "always";
      wantedBy = [ "default.target" ];
    };
  };
}
