{pkgs}:
pkgs.mkShell {
  name = "aplin devShell";
  nativeBuildInputs = with pkgs; [
    pkg-config
    dbus
    kdePackages.kdialog
  ];
  buildInputs = with pkgs; [
    cargo
    rustc
    rust-analyzer
    rustfmt
    clippy

    # Tools
    scdoc
    cargo-audit
    cargo-xbuild
    cargo-deny
  ];
}
