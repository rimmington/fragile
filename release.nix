{ nixpkgs ? <nixpkgs>
, systems ? [ "x86_64-linux" ]
, fragile ? ./. }:

let
  lib = (import nixpkgs {}).lib;
in lib.genAttrs systems (system:
  let
    pkgs = import nixpkgs { inherit system; };
  in {
    build = pkgs.callPackage fragile {};
  }
)
