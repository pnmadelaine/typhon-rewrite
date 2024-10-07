nixpkgs:
let
  self = rec {
    systems = [
      "x86_64-linux"
      "aarch64-linux"
    ];

    nixpkgsLib = import "${nixpkgs}/lib";

    pkgs = nixpkgsLib.genAttrs systems (system: import nixpkgs { inherit system; });

    unionOfDisjoint =
      x: y:
      if builtins.intersectAttrs x y == { } then
        x // y
      else
        throw "unionOfDisjoint: intersection is not empty";

    mkScope = scope: fn: if scope == null then fn else lib: { ${scope} = fn lib; };

    importList =
      scope: list:
      mkScope scope (
        nixpkgsLib.foldr (
          path: fn: lib:
          unionOfDisjoint (import path self lib) (fn lib)
        ) (_: { }) list
      );

    importPath =
      scope: path:
      importList scope (
        nixpkgsLib.mapAttrsToList (x: _: /${path}/${x}) (
          builtins.removeAttrs (builtins.readDir path) [
            "default.nix"
            "utils.nix"
            "README.md"
          ]
        )
      );

    jqJsonToBashArray = ''to_entries | .[] | "[" + (.key | @sh) + "]=" + (.value | tostring | @sh)'';
  };
in
self
