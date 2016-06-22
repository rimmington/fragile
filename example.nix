{nixpkgs ? <nixpkgs>}:

with import nixpkgs {};

stdenv.mkDerivation {
  name = "example-test";
  src = ./.;
  passAsFile = [ "config" ];
  config = ''
    {...}:
    {
      services.cron.enable = true;
    }
  '';
  buildCommand = ''
    export NIX_PATH=nixpkgs=${nixpkgs}
    /var/setuid-wrappers/sudo -n /run/current-system/sw/bin/fragile $configPath systemctl status > $out
    grep cron.service $out || (echo "Cron didn't start"; false)
  '';
}
