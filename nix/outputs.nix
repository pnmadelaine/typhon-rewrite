rec {
  mkLib = import ../lib;

  mkPackage = import ./package.nix;

  nixosModules = rec {
    typhon = import ./module.nix;
    default = typhon;
  };

  sources = import ./sources.nix;

  lib = mkLib sources.nixpkgs;

  devShells = lib.eachSystem (system: {
    default = import ./shell.nix { inherit system; };
  });

  packages = lib.eachSystem (system: rec {
    typhon = mkPackage { inherit system; };
    typhon-cli = typhon.cli;
    typhon-doc = typhon.doc;
    default = typhon;
  });

  jobs = import ./jobs.nix;
}
