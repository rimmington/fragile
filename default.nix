{rustPlatform, stdenv}:

# rustPlatform.buildRustPackage {
#   name = "fragile";
#   src = ./.;
#   depsSha256 = "1laisxsm25rln5fz29igpk8rw1j92lj8ddki7lxyd9384g45s082";
# }

stdenv.mkDerivation {
  name = "fragile";
  src = ./.;
  buildPhase = ''
    mkdir -p $out/bin
    ${rustPlatform.rustc}/bin/rustc -o $out/bin/fragile src/main.rs
  '';
  installPhase = "true";
}
