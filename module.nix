{config, lib, pkgs, ...}:

with lib;

let
  cfg = config.services.fragile;
  fragile = pkgs.callPackage ./. { suPath = "${config.security.wrapperDir}/su"; };
in {
  options = {
    services.fragile.enable = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Whether to enable fragile, placing it in the system packages.
        This may have security implications.
      '';
    };

    services.fragile.permitNixbldSudo = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether to permit passwordless sudo access to fragile for users in the
        nixbld group.
      '';
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ fragile ];

    security.sudo.extraConfig = mkIf cfg.permitNixbldSudo ''

      # services.fragile.permitNixbldSudo
      Cmnd_Alias FRAGILE = ${fragile}/bin/fragile
      Defaults!FRAGILE env_keep += NIX_PATH
      %nixbld ALL = NOPASSWD: FRAGILE
    '';
  };
}
