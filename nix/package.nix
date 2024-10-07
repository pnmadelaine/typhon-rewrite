let
  root = ./..;
  mkCargoNix =
    pkgs:
    builtins.mapAttrs (_: crate: crate.build)
      (pkgs.callPackage /${root}/nix/cargo.nix {
        defaultCrateOverrides = pkgs.defaultCrateOverrides // {
          leptos_config = attrs: { CARGO_CRATE_NAME = "leptos_config"; };
          typhon-core = attrs: { SYSTEM = pkgs.system; };
        };
      }).workspaceMembers;
in
{
  system ? builtins.currentSystem or "unknown-system",
  pkgs ? import (import /${root}/nix/sources.nix).nixpkgs { inherit system; },
}:
let
  version = pkgs.lib.trim (builtins.readFile /${root}/version.txt);
  cargoNixHost = mkCargoNix pkgs;
  cargoNixBuild = mkCargoNix pkgs.pkgsBuildBuild;
  cargoNixWasm = mkCargoNix pkgs.pkgsCross."wasm32-unknown-none";
  inherit (cargoNixHost) typhon-server;
  inherit (cargoNixBuild) typhon-pack;
  typhon-app =
    ((mkCargoNix pkgs.pkgsCross."wasm32-unknown-none").typhon-app.override {
      features = [ "hydrate" ];
    }).lib;
  typhon-cli = pkgs.stdenv.mkDerivation {
    pname = "typhon-cli";
    inherit version;
    dontUnpack = true;
    installPhase = ''
      mkdir -p $out/bin
      cp ${cargoNixHost.typhon-cli}/bin/ty $out/bin
    '';
  };
  typhon-doc = pkgs.stdenv.mkDerivation {
    pname = "typhon-doc";
    inherit version;
    src = /${root}/doc;
    nativeBuildInputs = [ pkgs.mdbook ];
    buildPhase = "mdbook build";
    installPhase = "cp -r book $out";
  };
in
pkgs.stdenv.mkDerivation {
  pname = "typhon";
  inherit version;
  dontUnpack = true;
  installPhase = ''
    mkdir -p $out/{bin,site}
    cp -r ${typhon-server}/bin/typhon-server $out/bin/typhon
    ${typhon-pack}/bin/typhon-pack $out/site/pkg ${typhon-app}/lib/*.wasm
  '';
  passthru = {
    cli = typhon-cli;
    doc = typhon-doc;
  };
  meta = {
    description = "Nix-based continuous integration";
    mainProgram = "typhon";
    homepage = "https://typhon-ci.org/";
    license = pkgs.lib.licenses.agpl3Plus;
    platforms = [
      "aarch64-linux"
      "x86_64-linux"
    ];
  };
}
