{
  description = "aufind";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        pname = cargoToml.package.name;
        version = cargoToml.package.version;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit pname version;

          src = ./.;
          checkFlags = [
            # This tests fails when ran outside of the git repo environment (and rightfully so),
            # and this is exactly what nix build does
            "--skip=tests::test_gitignore"
          ];
          
          cargoLock = {
            lockFile = ./Cargo.lock;
            # Needed when consuming dependencies from git. I don't like it, but can live with it.
            outputHashes = {
              "cdfs-0.2.4-pre" = "sha256-a6blUljHEnJdNzxfIgY4VpOMVeejHkcKC3ZCEG3zv2A=";
            };
          };

          meta = with pkgs.lib; {
            description = pname;
            license = licenses.mit;
          };
        };

        # For development
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
          ];
        };
      }
    );
}
