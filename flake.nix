{
  description = "DevShell for Rust.";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    devShells.${system}.default = pkgs.mkShell {
      # Add packages here.
      buildInputs = with pkgs; [
        cargo
        rust-analyzer
        rustc
        rustfmt
      ];

      # Shell hooks.
      shellHook = ''
        echo "Entering the development environment!"
        rustc --version
        cargo --version
      '';
    };
  };
}
