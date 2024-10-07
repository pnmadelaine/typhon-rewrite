let
  sources = import ./sources.nix;
in
{
  system ? builtins.currentSystem or "unknown-system",
  pkgs ? import sources.nixpkgs { inherit system; },
}:
let
  rustfmt-nightly = pkgs.rustfmt.override (attrs: {
    asNightly = true;
  });
in
pkgs.mkShell {
  name = "typhon-devshell";
  packages = with pkgs; [
    # rust:
    (pkgs.writeShellScriptBin "rustfmt" "${rustfmt-nightly}/bin/rustfmt --config-path $ROOT/.config")
    cargo
    cargo-binutils
    llvmPackages.bintools # lld
    openssl.dev
    pkg-config
    rust-analyzer
    rustc
    # javascript:
    node2nix
    nodejs
    npm-check-updates
    # nix:
    (import sources.crate2nix { inherit pkgs; })
    # misc:
    (import ./treefmt.nix { inherit system; }).config.build.wrapper
    bubblewrap
    diesel-cli
    mdbook
    postgresql
  ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  SYSTEM = system;
  shellHook = ''
    export ROOT="$(pwd)"
    export PATH="$ROOT/.scripts:$PATH"
    export WORKSPACE="$ROOT/workspace"
    export CONFIG="$ROOT/.config/taplo.toml"
    export DATABASE_URL="postgres://localhost:5432/typhon"
    export MANIFEST="$WORKSPACE/Cargo.toml"
    export PGDATA="/tmp/typhon/.postgres"
    export SOURCES_PATH="$ROOT/nix/sources.json"
    git config --local core.hooksPath "$ROOT/.scripts/git_hooks"
    mkdir -p $(dirname "$PGDATA")
    ${pkgs.figlet}/bin/figlet -f roman typhon
  '';
}
