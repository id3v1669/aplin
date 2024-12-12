{
  pkgs,
  fenix
}:let
  android = {
    platforms = [ "34" ];
    platformTools = "33.0.3";
    buildTools = [ "30.0.3" ];
  };

  sdkArgs = {
    platformVersions = android.platforms;
    platformToolsVersion = android.platformTools;
    buildToolsVersions = android.buildTools;
    includeNDK = true;
  };
  androidComposition = pkgs.androidenv.composeAndroidPackages sdkArgs;
  androidHome = "${androidComposition.androidsdk}/libexec/android-sdk";
in
pkgs.mkShell{
  name = "aplin devShell";
  nativeBuildInputs = with pkgs; [
    pkg-config
    dbus
  ];
  buildInputs = with pkgs; let 
  androidTargets = [
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
    "i686-linux-android"
  ];
  fenixPkgs = fenix.packages.${pkgs.system};
  rust-toolchain = with fenixPkgs.stable; fenixPkgs.combine [
    cargo
    rustc
    rust-analyzer
    rustfmt
    clippy
    (pkgs.lib.forEach androidTargets (target: fenixPkgs.targets."${target}".stable.rust-std))
  ];
  in [
    rust-toolchain
    #androidComposition.androidsdk

    jdk17

    # Tools
    scdoc
    #android-tools
    #cargo-apk
    #cargo-ndk
    cargo-audit
    cargo-xbuild
    cargo-deny
  ];
  #RUST_TARGETS = [ "aarch64-linux-android" "armv7-linux-androideabi" "x86_64-linux-android" "i686-linux-android" "x86_64-unknown-linux-gnu" ];
  #ANDROID_HOME = androidHome;
  #ANDROID_SDK_ROOT = androidHome;
  #ANDROID_NDK_ROOT = "${androidHome}/ndk-bundle";
}