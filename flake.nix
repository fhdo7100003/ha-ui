{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        ha-ui =
          let
            cargoToml = builtins.fromTOML (builtins.readFile "${self}/Cargo.toml");
            version = cargoToml.package.version;
            pname = cargoToml.package.name;
            craneLib = crane.mkLib pkgs;
          in
          craneLib.buildPackage {
            inherit pname version;
            src = self;
            buildInputs = with pkgs; [
              wayland
              libxkbcommon
              libinput
              fontconfig
              stdenv.cc.cc.lib
            ];
            outputHashes = {
              "git+https://github.com/pop-os/cosmic-text.git#1f4065c1c3399efad58841082212f7c039b58480" = "sha256-aNzLtD8ma+EJF4d6liThigJvQRsGWC2Zng8t2em02Mo=";
            };
            cargoSha256 = pkgs.lib.fakeSha256;
            nativeBuildInputs = with pkgs; [
              autoPatchelfHook
              installShellFiles
              pkg-config
            ];

            runtimeDependencies = with pkgs; [
              libglvnd # For libEGL
              wayland # winit->wayland-sys wants to dlopen libwayland-egl.so
              # for running in X11
              xorg.libX11
              xorg.libXcursor
              xorg.libxcb
              xorg.libXi
              libxkbcommon
              # for vulkan backend
              vulkan-loader
            ];
          };
      in
      {
        packages = {
          inherit ha-ui;
          default = ha-ui;
        };
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            libxkbcommon
            freetype
            fontconfig
            libinput
            glib
          ];
          LD_LIBRARY_PATH =
            with pkgs;
            lib.strings.makeLibraryPath [
              wayland
              libglvnd
              libxkbcommon
              glib
            ];
        };
      }
    );
}
