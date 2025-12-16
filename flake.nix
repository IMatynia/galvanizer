
{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, fenix }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };

    rust = fenix.packages.${system}.stable.withComponents [
      "cargo"
      "rustc"
      "rustfmt"
      "clippy"
      "rust-src"
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        rust
        pkgs.just
        pkgs.neovim
        pkgs.bacon
        pkgs.zsh
        pkgs.cargo-tarpaulin
      ];

      nativeBuildInputs = [
        pkgs.pkg-config
        pkgs.gcc
      ];
    };
  };
}

