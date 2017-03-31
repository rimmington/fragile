{rustPlatform, stdenv, nix, systemd, coreutils, eject, findutils, suPath ? "/run/wrappers/bin/su"}:

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
      --replace '"rm"' '"${coreutils}/bin/rm"' \
      --replace '"su"' '"${suPath}"'
  '';
  depsSha256 = "0azj2gmw1zy08dil7y2jclri1n52qcyn9hb88hln768ffj4bd2pp";
  shellHook = "unset SSL_CERT_FILE";  # https://github.com/NixOS/nixpkgs/issues/13744
}
