{
  description = "aplin";

  inputs = { 
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
    fenix = {
      url = "github:nix-community/fenix";
      #inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ 
  { self
  , nixpkgs
  , systems
  , fenix
  , ... 
  }:
  let
    eachSystem = nixpkgs.lib.genAttrs (import systems);

    pkgsFor = (system: import nixpkgs {
      inherit system;
      overlays = [
        fenix.overlays.default
      ];
      config = {
        android_sdk.accept_license = true;
        allowUnfree = true;
      };
    });
  in 
  { 
    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix { fenix = fenix; };
    });
  };
}