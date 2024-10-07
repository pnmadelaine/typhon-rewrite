nixpkgs:
let
  utils = import ./utils.nix nixpkgs;
  self = utils.importPath null ./. self;
in
self
