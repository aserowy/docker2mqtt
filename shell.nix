{ nixpkgs ? import <nixpkgs> { } }:

let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner = "NixOS";
    repo = "nixpkgs";
    rev = "1fe6ed37fd9beb92afe90671c0c2a662a03463dd";
    sha256 = "1daa0y3p17shn9gibr321vx8vija6bfsb5zd7h4pxdbbwjkfq8n2";
  };
  pkgs = import pinnedPkgs {
    overlays = [ (import rustOverlay) ];
  };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rust-bin.stable.latest.default
    rust-analyzer
  ];

  RUST_BACKTRACE = 1;
}
