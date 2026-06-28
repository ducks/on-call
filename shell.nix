{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    docker
    docker-compose
  ];

  RUST_BACKTRACE = 1;
}
