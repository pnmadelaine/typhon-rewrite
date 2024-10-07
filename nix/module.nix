let
  root = ./..;
in
{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib)
    mkEnableOption
    mkIf
    mkOption
    types
    ;

  cfg = config.services.typhon;
in
{
  options.services.typhon = {
    enable = mkEnableOption "typhon";
    package = mkOption {
      type = types.package;
      description = "Which package to use for the Typhon instance";
      default = import /${root}/default.nix { inherit (pkgs) system; };
    };
    home = mkOption {
      type = types.str;
      default = "/var/lib/typhon";
      description = "Home directory for the Typhon instance";
    };
  };

  config = mkIf cfg.enable {
    users.users.typhon = {
      home = cfg.home;
      group = "typhon";
      createHome = true;
      isSystemUser = true;
    };
    users.groups.typhon = { };

    systemd.services.typhon-init = {
      description = "Typhon init";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = pkgs.writeShellScript "typhon-init" ''
          echo "TODO"
        '';
        RemainAfterExit = true;
        Type = "oneshot";
      };
    };

    systemd.services.typhon = {
      description = "Typhon service";
      wantedBy = [ "multi-user.target" ];
      path = [
        pkgs.bubblewrap
        pkgs.git
        pkgs.nix
        pkgs.openssh
      ];
      serviceConfig = {
        ExecStart = pkgs.writeShellScript "typhon-start" ''
          cd ${cfg.home}
          echo "TODO"
        '';
        Type = "simple";
        User = "typhon";
        Group = "typhon";
      };
      requires = [ "typhon-init.service" ];
      after = [ "typhon-init.service" ];
    };
  };
}
