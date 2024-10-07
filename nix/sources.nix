builtins.mapAttrs (
  name:
  { lock, ... }:
  if lock.type == "git" then
    (builtins.fetchGit {
      inherit (lock) url ref rev;
    }).outPath
  else if lock.type == "github" then
    builtins.fetchTarball {
      url = "https://github.com/${lock.owner}/${lock.repo}/archive/${lock.rev}.tar.gz";
      inherit (lock) sha256;
    }
  else
    throw "Unrecognized lock type: ${lock.type}"
) (builtins.fromJSON (builtins.readFile ./sources.json))
