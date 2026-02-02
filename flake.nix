{
  description = "MoFA Studio - Rust UI with Python backends (simplified)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = { allowUnfree = true; };
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        python = pkgs.python312;

        runScript = pkgs.writeShellApplication {
          name = "run-mofa";
          runtimeInputs = [
            rustToolchain
            python
            pkgs.git
            pkgs.cmake
            pkgs.pkg-config
            pkgs.openssl
            pkgs.portaudio
          ];
          text = ''
            set -euo pipefail

            ROOT="''${MOFA_STUDIO_DIR:-$PWD}"
            if [ ! -d "$ROOT" ]; then
              echo "[MoFA][Nix] 无法找到源码目录 $ROOT" >&2
              exit 1
            fi

            export CARGO_HOME="''${MOFA_CARGO_HOME:-$ROOT/.cargo}"
            
            # Setup Python venv if needed
            VENV_DIR="''${MOFA_VENV_DIR:-$ROOT/.venv-mofa}"
            if [ ! -d "$VENV_DIR" ]; then
              echo "[MoFA][Nix] 创建 Python venv..."
              python3 -m venv "$VENV_DIR"
            fi
            source "$VENV_DIR/bin/activate"
            
            echo "[MoFA][Nix] 启动应用 (Python + Rust)..."
            cd "$ROOT"
            cargo run --release --bin mofa-studio
          '';
        };
      in
      {
        packages.default = runScript;
        apps.default = {
          type = "app";
          program = "${runScript}/bin/run-mofa";
        };
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            python
            pkgs.git
            pkgs.cmake
            pkgs.pkg-config
            pkgs.openssl
            pkgs.portaudio
          ];
        };
      });
}
