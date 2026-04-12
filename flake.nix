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
      forAllSystems = f: nixpkgs.lib.genAttrs systems f;
    in {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          toolchain = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; };
        in {
          default = rustPlatform.buildRustPackage {
            pname = "hydra-banner";
            version = "0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
          getmd = pkgs.stdenvNoCC.mkDerivation {
            name = "getmd";
            src = ./scripts/getmd;
            dontUnpack = true;
            installPhase = "install -D $src $out/bin/getmd";
          };
          glb = pkgs.writeShellApplication {
            name = "glb";
            runtimeInputs = [ pkgs.hydra-check pkgs.jq ];
            text = builtins.readFile ./scripts/glb;
          };
          docker = pkgs.dockerTools.buildImage {
            name = "ghcr.io/miniharinn/hydra-banner";
            tag = "latest";
            copyToRoot = [ self.packages.${system}.default ];
            config = {
              Cmd = [ "${self.packages.${system}.default}/bin/hydra-banner" ];
              Env = [ "PORT=3000" ];
              ExposedPorts."3000/tcp" = { };
            };
          };
        }
      );

      apps = forAllSystems (system:
        let pkg = name: { type = "app"; program = "${self.packages.${system}.${name}}/bin/${name}"; };
        in {
          getmd = pkg "getmd";
          glb = pkg "glb";
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = [ toolchain pkgs.watchexec ];
            RUST_LOG = "hydra_banner=debug,tower_http=debug";
            shellHook = ''echo "dev server: ./scripts/dev"'';
          };
        }
      );
    };
}
