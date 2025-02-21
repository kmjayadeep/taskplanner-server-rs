{
  description = "A Rust project with Docker image build support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            openssl
            sqlx-cli
          ];

          shellHook = ''
            exec zsh
          '';
        };
      }
    );
}
