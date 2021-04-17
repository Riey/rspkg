let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rust = (pkgs.rustChannelOf { channel = "stable"; }).rust
  .override {
    targets = [
      "x86_64-unknown-linux-gnu"
      "wasm32-unknown-unknown"
    ];
    extensions = ["rust-src"];
  };
in
with pkgs;
mkShell {
  name = "rspkg-shell";

  buildInputs = [
  ];
  nativeBuildInputs = [
    rust
  ];
}

