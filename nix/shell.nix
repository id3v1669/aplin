{pkgs}:
pkgs.mkShell {
  name = "aplin devShell";
  nativeBuildInputs = with pkgs; [
    # Compilers
    cargo
    rustc
    scdoc

    # build Deps
    pkg-config
    dbus

    # Tools
    cargo-audit
    cargo-deny
    clippy
    rust-analyzer
    rustfmt
  ];
}