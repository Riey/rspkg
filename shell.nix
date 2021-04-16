{ nixpkgs ? <nixpkgs>, }:
let
  pkgs = import nixpkgs {};
in
pkgs.mkShell {
  name = "rust-pager-shell";
  nativeBuildInput = with pkgs; [ rustc cargo ];
}

