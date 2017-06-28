{config, lib, pkgs, ...}:

with lib;

let
  cfg = config.programs.fragile;
  fragile = pkgs.callPackage ./. { suPath = "${config.security.wrapperDir}/su"; };
in {
  options = {
    programs.fragile.setuidWrapper = mkOption {
      type = types.bool;
      default = false;
      description = ''
        Whether to add a setuid wrapper for fragile, allowing users in the
        specified setuidGroup to execute fragile as root (ie. at all).
      '';
    };

    programs.fragile.setuidGroup = mkOption {
      type = types.string;
      default = "nixbld";
      description = ''
        The group that is allowed to execute the setuid wrapper for fragile.
      '';
    };
  };

  config = mkIf cfg.setuidWrapper {
    security.wrappers.fragile = {
      source = "${fragile}/bin/fragile";
      owner = "root";
      group = cfg.setuidGroup;
      setuid = true;
      permissions = "u+rx,g+x";
    };
  };
}
