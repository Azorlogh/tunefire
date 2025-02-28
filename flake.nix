{
  description = "Rust devshell.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        devShells.default = with pkgs;
          mkShell rec {
            buildInputs = [
              openssl
              rust-bin.beta.latest.default
              sqlite
              pkg-config
              alsa-lib
              gdk-pixbuf
              pango
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              xorg.libxcb
              libxkbcommon
              vulkan-loader
              wayland
              cairo
              atk
              gtk3
              protobuf
            ];

            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
            '';
          };
      }
    );
}
