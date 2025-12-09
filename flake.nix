{
  description = "PMAT - Pragmatic Multi-language Agent Toolkit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Build inputs needed for compilation
        buildInputs = with pkgs; [
          openssl
          pkg-config
          zlib
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Development tools
        devTools = with pkgs; [
          cargo-watch
          cargo-audit
          cargo-deny
          cargo-llvm-cov
          cargo-mutants
          wasm-pack
          wasm-bindgen-cli
          trunk
          nodejs_20
          jq
          heaptrack
        ];

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs ++ devTools ++ [ rustToolchain ];

          shellHook = ''
            echo "PMAT Development Environment"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build         - Build the project"
            echo "  cargo test          - Run tests"
            echo "  cargo llvm-cov      - Generate coverage report"
            echo "  cargo mutants       - Run mutation testing"
            echo "  pmat popper-score   - Self-assess scientific rigor"
          '';

          RUST_BACKTRACE = "1";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Package definition
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "pmat";
          version = "2.210.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = with pkgs; [ pkg-config ];
          inherit buildInputs;

          # Skip tests during nix build (run separately)
          doCheck = false;

          meta = with pkgs.lib; {
            description = "Pragmatic Multi-language Agent Toolkit";
            homepage = "https://github.com/paiml/paiml-mcp-agent-toolkit";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        # WASM dashboard package
        packages.dashboard = pkgs.stdenv.mkDerivation {
          pname = "pmat-dashboard";
          version = "2.210.0";
          src = ./crates/pmat-dashboard;

          nativeBuildInputs = [ pkgs.wasm-pack rustToolchain ];

          buildPhase = ''
            export HOME=$TMPDIR
            wasm-pack build --target web --release
          '';

          installPhase = ''
            mkdir -p $out
            cp -r pkg/* $out/
          '';
        };
      }
    );
}
