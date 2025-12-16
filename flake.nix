
{
  description = "Rust development template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  };

  outputs = { self, nixpkgs, ... }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in
  {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        rustc
        cargo
        just
        neovim
        bacon
        zsh
        cargo-tarpaulin
      ];

      nativeBuildInputs = with pkgs; [
        pkg-config
        gcc
      ];
    };
  };
}

