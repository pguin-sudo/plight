{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {
        config,
        self',
        pkgs,
        lib,
        system,
        ...
      }: let
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        msrv = cargoToml.package.rust-version;

        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };

        runtimeDeps = with pkgs; [
          pipewire
          dbus
          udev
          libxcb
          xorg.libX11
          stdenv.cc.cc.lib
        ];

        buildDeps = with pkgs; [
          pkg-config
          llvmPackages.clang
          rustPlatform.bindgenHook
        ];

        devDeps = with pkgs; [
          rust-analyzer
          cargo-edit
          lldb
          gdb
        ];

        mkPackage = features:
          (pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
          }).buildRustPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildFeatures = features;
            buildInputs = runtimeDeps;
            nativeBuildInputs = buildDeps;
          };

        mkDevShell = rustToolchain:
          pkgs.mkShell {
            shell = "${pkgs.zsh}/bin/zsh";

            shellHook = ''
              export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
               echo "Activated Rust development shell with $(rustc --version)"

              if [ -n "$BASH" ]; then
                exec ${pkgs.zsh}/bin/zsh
              fi
            '';
            buildInputs = runtimeDeps;
            nativeBuildInputs =
              buildDeps
              ++ devDeps
              ++ [
                rustToolchain
                pkgs.zsh
              ];

            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            RUST_BACKTRACE = "full";
          };
      in {
        packages = {
          plight = mkPackage "";
          default = self'.packages.plight;
        };

        devShells = {
          stable = mkDevShell pkgs.rust-bin.stable.latest.default;
          nightly = mkDevShell (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default));
          msrv = mkDevShell pkgs.rust-bin.stable.${msrv}.default;
          default = self'.devShells.stable;
        };
      };
    };
}
