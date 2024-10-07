let
  sources = import ./sources.nix;
in
{
  system ? builtins.currentSystem or "unknown-system",
  pkgs ? import sources.nixpkgs { inherit system; },
}:
(import sources.treefmt-nix).evalModule pkgs {
  projectRootFile = "version.txt";
  settings.global.excludes = [
    "nix/cargo.nix"
    "nix/node2nix/*"
  ];

  # Rust
  programs.rustfmt.enable = true;
  programs.rustfmt.package = pkgs.rustfmt.override (attrs: {
    asNightly = true;
  });
  settings.formatter.rustfmt.options = [
    "--config-path"
    ".config/rustfmt.toml"
  ];

  # TOML
  programs.taplo.enable = true;
  programs.taplo.package = pkgs.writeShellScriptBin "taplo" "${pkgs.taplo}/bin/taplo format $@";
  settings.formatter.taplo.options = pkgs.lib.mkForce [
    "-c"
    ".config/taplo.toml"
  ];

  # SQL
  programs.sqlfluff.enable = true;
  programs.sqlfluff.dialect = "postgres";

  # Nix
  programs.nixfmt.enable = true;

  # Leptos
  programs.leptosfmt.enable = true;
}
