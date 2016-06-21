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
  depsSha256 = "1wjnrsxagb1v03spdmfrfdas59a53qacj8i1gibwqrvqbmxw4wmj";
}
