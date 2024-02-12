{ pkgs ? import <nixpkgs> { } }:

with pkgs;
let
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz") { };
in
mkShell rec {
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    (with fenix; combine [
        stable.cargo stable.clippy stable.rust-src stable.rustc stable.rustfmt targets.wasm32-unknown-unknown.stable.rust-std
     ])
    udev alsa-lib vulkan-loader
    clang mold
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
    libxkbcommon wayland
    openssl wasm-bindgen-cli
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}

# cargo build --release --target wasm32-unknown-unknown && wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "xrds-teachable-machine" ./target/wasm32-unknown-unknown/release/xrds-teachable-machine.wasm

