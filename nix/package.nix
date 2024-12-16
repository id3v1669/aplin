{ lib
, rustPlatform
, makeWrapper
, pkg-config
, pkgs
}:
rustPlatform.buildRustPackage rec {

  pname = "aplin";
  version = "0.0.1";

  src = lib.cleanSource ../.;

  cargoLock.lockFile = "${src}/Cargo.lock";

  nativeBuildInputs = [ pkgs.pkg-config ];

  buildInputs = [ pkgs.dbus ];
}