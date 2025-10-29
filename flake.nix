{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    rust-overlay,
    ...
  }:
    flake-parts.lib.mkFlake {
      inherit inputs;
    } {
      systems = ["x86_64-linux"];

      perSystem = {
        config,
        pkgs,
        system,
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        runtimeDeps = with pkgs; [
          pipewire
          dbus
          udev
          libxcb
          stdenv.cc.cc.lib
        ];

        buildDeps = with pkgs; [
          pkg-config
          llvmPackages.clang
          rustPlatform.bindgenHook
        ];

        nativeBuildDeps = with pkgs; [
          pkg-config
          llvmPackages.clang
        ];
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-edit
            lldb
            rust-analyzer
          ];

          buildInputs = runtimeDeps;
          nativeBuildInputs = nativeBuildDeps;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.clang.version}/include";

          shellHook = ''
            export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
            echo "Rust: $(rustc --version)"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit (cargoToml.package) name version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          buildInputs = runtimeDeps;
          nativeBuildInputs = buildDeps;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      };
    };
}
