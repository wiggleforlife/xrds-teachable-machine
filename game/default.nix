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
    fenix.latest.toolchain #TODO add targets.wasm32-unknown-unknown.stable.rust-std
    udev alsa-lib vulkan-loader
    clang lld mold # TODO lld/mold sometimes cannot find dylibs
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr
    libxkbcommon wayland
    openssl
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

  # shellHook = "rust-rover .";
}
