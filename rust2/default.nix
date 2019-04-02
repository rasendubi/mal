{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  name = "mal-rust";
  buildInputs = [
    pkgs.rust.rustc
    pkgs.rust.cargo
  ];
}
