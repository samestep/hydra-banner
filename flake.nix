{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
      getmd = system:
        let pkgs = import nixpkgs { inherit system; };
        in pkgs.stdenvNoCC.mkDerivation {
          name = "getmd";
          src = ./scripts/getmd;
          dontUnpack = true;
          installPhase = "install -D $src $out/bin/getmd";
        };
    in {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
          toolchain = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; };
          rustPackage = rustPlatform.buildRustPackage {
            pname = "hydra-banner";
            version = "0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
        in {
          default = rustPackage;
          getmd = getmd system;
          docker = pkgs.dockerTools.buildImage {
            name = "ghcr.io/miniharinn/hydra-banner";
            tag = "latest";
            copyToRoot = [ rustPackage ];
            config = {
              Cmd = [ "${rustPackage}/bin/hydra-banner" ];
              Env = [ "PORT=3000" ];
              ExposedPorts = {
                "3000/tcp" = { };
              };
            };
          };
        }
      );

      apps = forAllSystems (system: {
        getmd = { type = "app"; program = "${getmd system}/bin/getmd"; };
      });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              toolchain
              pkgs.watchexec
            ];
            RUST_LOG = "hydra_banner=debug,tower_http=debug";
            shellHook = ''
              echo "dev server: ./scripts/dev"
            '';
          };
        }
      );
    };
}
