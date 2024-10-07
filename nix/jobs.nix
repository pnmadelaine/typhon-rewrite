let
  sources = import ./sources.nix;
  system = "x86_64-linux";
  pkgs = import sources.nixpkgs { inherit system; };
  typhon = import ./package.nix { inherit system; };
in
{
  "${system}.build" = typhon;
  "${system}.cli" = typhon.cli;
  "${system}.doc" = typhon.doc;
  "${system}.formatted" = (import ./treefmt.nix { inherit system; }).config.build.check ./..;
  "${system}.nixosTest" = import ./nixosTest.nix { inherit system; };
}
