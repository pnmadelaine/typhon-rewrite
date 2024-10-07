# Installation

## Nix requirements

Typhon requires the experimental feature "nix-command" to be enabled.

## NixOS

The only supported way to install Typhon is on NixOS via the exposed module.

### Example

Here is a sample NixOS module that deploys a Typhon instance:

```nix
{ pkgs, ... }:

{
  imports = [ (<typhon> + "/nix/module.nix") ]; # import the module in your preferred way

  # enable the "nix-command" feature 
  nix.settings.experimental-features = [ "nix-command" ];

  # enable Typhon
  services.typhon = {
    enable = true;

    # TODO
  };

  # configure nginx
  services.nginx = {
    enable = true;
    forceSSL = true;
    enableACME = true;
    virtualHosts."etna.typhon-ci.org" = {
      locations."/" = {
        proxyPass = "http://localhost:3000";
        recommendedProxySettings = true;
      };
    };
  };
}
```


### Options

Here is a list of options exposed by the NixOS module.

Mandatory:

- `services.typhon.enable`: a boolean to activate the Typhon instance.

Optional:

- `services.typhon.home`: a string containing the home directory of the Typhon
  instance.
- `services.typhon.package`: a derivation to override the package used for the
  Typhon instance.
