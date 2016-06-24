{pkgs, ...}:

let
  fragile = pkgs.callPackage ./. {};
in {
  environment.systemPackages = [ fragile ];
  security.sudo.extraConfig = ''
    Cmnd_Alias FRAGILE = ${fragile}/bin/fragile
    Defaults!FRAGILE env_keep += NIX_PATH
    %nixbld ALL = NOPASSWD: FRAGILE
  '';
}
