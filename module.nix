{pkgs, ...}:

let
  fragile = pkgs.callPackage ./. {};
in {
  environment.systemPackages = [ fragile ];
  security.sudo.extraConfig = ''
    %nixbld ALL = NOPASSWD: ${fragile}/bin/fragile
  '';
}
