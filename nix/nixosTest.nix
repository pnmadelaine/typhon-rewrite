let
  root = ./..;
  sources = import /${root}/nix/sources.nix;
  module = import ./module.nix;
in
{
  system ? builtins.currentSystem or "unknown-system",
  pkgs ? import sources.nixpkgs { inherit system; },
}:
pkgs.testers.nixosTest (
  { pkgs, ... }:
  {
    name = "typhon-test";

    nodes = {
      typhon =
        { ... }:
        {
          nix.settings.experimental-features = [
            "nix-command"
            "flakes"
          ];
          imports = [ module ];
          services.typhon.enable = true;
          services.nginx = {
            enable = true;
            virtualHosts."example.com" = {
              locations."/" = {
                proxyPass = "http://localhost:3000";
                recommendedProxySettings = true;
              };
            };
          };
        };
    };

    testScript =
      { nodes, ... }:
      ''
        typhon.start()
        typhon.wait_for_unit("default.target")

        with subtest("Wait for Typhon"):
            typhon.wait_for_unit("typhon.service")

        with subtest("Wait for nginx"):
            typhon.wait_for_unit("nginx.service")
      '';
  }
)
