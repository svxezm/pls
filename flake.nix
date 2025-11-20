{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/release-25.05";

  outputs = { nixpkgs, ... }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in
  {
    packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
      pname = "pls";
      version = "0.1.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
    };
  };
}
