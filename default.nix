{rustPlatform, stdenv, nix, systemd, coreutils, eject, findutils}:

rustPlatform.buildRustPackage {
  name = "fragile";
  src = ./.;
  postPatch = ''
    substituteInPlace src/main.rs \
      --replace '"nix-env"' '"${nix.out}/bin/nix-env"' \
      --replace '"systemctl"' '"${systemd}/bin/systemctl"' \
      --replace '"machinectl"' '"${systemd}/bin/machinectl"' \
      --replace '"nsenter"' '"${eject}/bin/nsenter"' \
      --replace '"find"' '"${findutils}/bin/find"' \
      --replace '"umount"' '"${eject}/bin/umount"' \
      --replace '"mountpoint"' '"${eject}/bin/mountpoint"' \
      --replace '"rm"' '"${coreutils}/bin/rm"'
  '';
  depsSha256 = "0mjazra3b6z128z5baq8hmq63b56iamh30qjn3ifzhx5znwq1vqi";
  shellHook = "unset SSL_CERT_FILE";  # https://github.com/NixOS/nixpkgs/issues/13744
}
