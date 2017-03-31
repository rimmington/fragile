{nixpkgs ? <nixpkgs>, pkgs ? import <nixpkgs> {}, stdenv ? pkgs.stdenv, lib ? pkgs.lib}:

let
  nixosPath = pkgs.nixosPath or <nixpkgs/nixos>;
  nixos = import nixosPath {};
  sudo = if lib.versionOlder "16.09" nixos.config.system.nixosRelease
    then "/run/wrappers/bin/sudo"
    else "/var/setuid-wrappers/sudo";

in stdenv.mkDerivation {
  name = "example-test";
  src = ./.;
  buildCommand = ''
    export NIX_PATH=nixpkgs=${nixpkgs}:nixpkgs/nixos=${nixosPath}
    ${sudo} -n /run/current-system/sw/bin/fragile ${./example-config.nix} systemctl status > $out
    grep cron.service $out || (echo "Cron didn't start"; false)
  '';
}
