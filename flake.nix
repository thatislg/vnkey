{
  description = "VnKey — Vietnamese input method (Fcitx5 & IBus)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        vnkey-engine = pkgs.rustPlatform.buildRustPackage {
          pname = "vnkey-engine";
          version = "1.0.1";

          src = ./vnkey-engine;

          cargoLock.lockFile = ./vnkey-engine/Cargo.lock;

          # Chỉ build static library
          buildPhase = ''
            cargo build --release
          '';

          installPhase = ''
            mkdir -p $out/lib $out/include
            cp target/release/libvnkey_engine.a $out/lib/
            cp src/vnkey-engine.h $out/include/ 2>/dev/null || true
          '';
        };

        vnkey-fcitx5 = pkgs.stdenv.mkDerivation {
          pname = "vnkey-fcitx5";
          version = "1.0.1";

          src = ./vnkey-fcitx5;

          nativeBuildInputs = with pkgs; [
            cmake
            pkg-config
          ];

          buildInputs = with pkgs; [
            fcitx5
          ];

          cmakeFlags = [
            "-DVNKEY_ENGINE_LIB_DIR=${vnkey-engine}/lib"
            "-DCMAKE_INSTALL_PREFIX=${placeholder "out"}"
            "-DFCITX_INSTALL_ADDONDIR=${placeholder "out"}/lib/fcitx5"
            "-DFCITX_INSTALL_PKGDATADIR=${placeholder "out"}/share/fcitx5"
          ];
        };

        vnkey-ibus = pkgs.stdenv.mkDerivation {
          pname = "vnkey-ibus";
          version = "1.0.1";

          src = ./vnkey-ibus;

          nativeBuildInputs = with pkgs; [
            cmake
            pkg-config
          ];

          buildInputs = with pkgs; [
            ibus
            glib
          ];

          cmakeFlags = [
            "-DVNKEY_ENGINE_LIB_DIR=${vnkey-engine}/lib"
            "-DCMAKE_INSTALL_PREFIX=${placeholder "out"}"
            "-DIBUS_LIBEXECDIR=${placeholder "out"}/libexec"
            "-DIBUS_COMPONENT_DIR=${placeholder "out"}/share/ibus/component"
          ];
        };

      in {
        packages = {
          inherit vnkey-engine vnkey-fcitx5 vnkey-ibus;
          default = vnkey-fcitx5;
        };
      }
    );
}
