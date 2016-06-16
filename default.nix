{rustPlatform, stdenv}:

rustPlatform.buildRustPackage {
  name = "fragile";
  src = ./.;
  depsSha256 = "15fsxd21810s6wiwhk4hzcpdb0a7pmjg7q30zyjgz83mnibslzcr";
}
