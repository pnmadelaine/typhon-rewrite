{
  cargo,
  lib,
  llvmPackages,
  openssl,
  pkg-config,
  rustPlatform,
  stdenv,
  system,
}:
stdenv.mkDerivation rec {
  pname = "typhon";
  version = "1.0.0";
  src = ./..;
  cargoDeps = rustPlatform.fetchCargoTarball {
    inherit src;
    name = "typhon-deps";
    sha256 = "sha256-rUln41mVowPfWCM21zIrRBUC2Knn7tGvH14OI8iJNHM=";
  };
  nativeBuildInputs = [
    cargo
    llvmPackages.bintools
    openssl
    pkg-config
    rustPlatform.cargoSetupHook
  ];
  buildPhase = ''
    export TYPHON_ROOT=$(pwd)
    bash .scripts/build release
  '';
  installPhase = ''
    mkdir -p $out/bin
    cp target/release/typhon-server $out/bin/typhon
    cp -r site/release $out/site
  '';
  SYSTEM = system;
  meta = {
    description = "Nix-based continuous integration";
    mainProgram = "typhon";
    homepage = "https://typhon-ci.org/";
    license = lib.licenses.agpl3Plus;
    maintainers = [ lib.maintainers.pnmadelaine ];
    platforms = [
      "aarch64-linux"
      "x86_64-linux"
    ];
  };
}
