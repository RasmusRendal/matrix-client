{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      fenix,
      nixpkgs,
    }:
    {

      devShells.x86_64-linux.default =
        let
          pkgs = import nixpkgs { system = "x86_64-linux"; };
        in
        pkgs.mkShell {
          packages = with pkgs; [
            fenix.packages.x86_64-linux.complete.toolchain

            # Slint
            eudev.out
            fontconfig.lib
            freetype.out
            libgcc.lib
            libinput.out
            libxkbcommon.out
            mesa.out
            wayland
            xorg.libX11
            xorg.libXcursor

            # matrix-sdk
            openssl
            pkg-config
            sqlite
          ];

          SLINT_LIVE_PREVIEW = "yes";

          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
            with pkgs;
            lib.makeLibraryPath [
              wayland
              libxkbcommon
              fontconfig
              libGL
              openssl
              sqlite
            ]
          }";
        };

      packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;

      packages.x86_64-linux.default = self.packages.x86_64-linux.hello;

    };
}
