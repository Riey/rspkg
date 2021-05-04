let
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust-overlay ]; };
  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" ];
    targets = [
      "x86_64-unknown-linux-gnu"
      "wasm32-unknown-unknown"
    ];
  };
in
pkgs.mkShell {
  name = "rspkg-shell";

  buildInputs = [
  ];
  nativeBuildInputs = [
    rust
  ];
}

