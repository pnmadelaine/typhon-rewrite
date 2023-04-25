# This file has been generated by node2nix 1.11.1. Do not edit!

{nodeEnv, fetchurl, fetchgit, nix-gitignore, stdenv, lib, globalBuildInputs ? []}:

let
  sources = {
    "@fontsource/roboto-4.5.8" = {
      name = "_at_fontsource_slash_roboto";
      packageName = "@fontsource/roboto";
      version = "4.5.8";
      src = fetchurl {
        url = "https://registry.npmjs.org/@fontsource/roboto/-/roboto-4.5.8.tgz";
        sha512 = "CnD7zLItIzt86q4Sj3kZUiLcBk1dSk81qcqgMGaZe7SQ1P8hFNxhMl5AZthK1zrDM5m74VVhaOpuMGIL4gagaA==";
      };
    };
    "@fontsource/roboto-mono-4.5.10" = {
      name = "_at_fontsource_slash_roboto-mono";
      packageName = "@fontsource/roboto-mono";
      version = "4.5.10";
      src = fetchurl {
        url = "https://registry.npmjs.org/@fontsource/roboto-mono/-/roboto-mono-4.5.10.tgz";
        sha512 = "KrJdmkqz6DszT2wV/bbhXef4r0hV3B0vw2mAqei8A2kRnvq+gcJLmmIeQ94vu9VEXrUQzos5M9lH1TAAXpRphw==";
      };
    };
  };
  args = {
    name = "typhon-webapp";
    packageName = "typhon-webapp";
    version = "1.0.0";
    src = ./..;
    dependencies = [
      sources."@fontsource/roboto-4.5.8"
      sources."@fontsource/roboto-mono-4.5.10"
    ];
    buildInputs = globalBuildInputs;
    meta = {
      license = "MIT";
    };
    production = true;
    bypassCache = true;
    reconstructLock = true;
  };
in
{
  args = args;
  sources = sources;
  tarball = nodeEnv.buildNodeSourceDist args;
  package = nodeEnv.buildNodePackage args;
  shell = nodeEnv.buildNodeShell args;
  nodeDependencies = nodeEnv.buildNodeDependencies (lib.overrideExisting args {
    src = stdenv.mkDerivation {
      name = args.name + "-package-json";
      src = nix-gitignore.gitignoreSourcePure [
        "*"
        "!package.json"
        "!package-lock.json"
      ] args.src;
      dontBuild = true;
      installPhase = "mkdir -p $out; cp -r ./* $out;";
    };
  });
}
