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
  depsSha256 = "1y0xm6j90x5m89kqmrqz87wn521x81hby6yii75bkjjlyymc0mq8";
  shellHook = "unset SSL_CERT_FILE";  # https://github.com/NixOS/nixpkgs/issues/13744
}
