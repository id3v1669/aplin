{
  description = "aplin";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);

    pkgsFor = system:
      import nixpkgs {
        inherit system;
      };
  in {
    packages = eachSystem (system: rec {
      default = aplin;
      aplin = (pkgsFor system).callPackage ./nix/package.nix {};
    });

    defaultPackage = eachSystem (system: self.packages.${system}.default);

    homeManagerModules.default = import ./nix/module.nix self;

    overlays.default = final: prev: {
      aplin = self.packages.${prev.system}.default;
    };

    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix {};
    });

    formatter.x86_64-linux = inputs.nixpkgs.legacyPackages.x86_64-linux.alejandra;
  };
}
