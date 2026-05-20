{
  nixConfig = {
    extra-substituters = [
      "https://claude-code.cachix.org"
    ];
    extra-trusted-public-keys = [
      "claude-code.cachix.org-1:YeXf2aNu7UTX8Vwrze0za1WEDS+4DuI2kVeWEE4fsRk="
    ];
    connect-timeout = 5;
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    claude-code = {
      url = "github:sadjow/claude-code-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      naersk,
      claude-code,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # ── Toolchain ─────────────────────────────────────────────
        rust = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        naersk' = pkgs.callPackage naersk {
          cargo = rust;
          rustc = rust;
        };

        # ── Build helper ──────────────────────────────────────────
        buildApp =
          { release }:
          naersk'.buildPackage {
            name = "rssh";
            src = ./.;
            inherit release;

            meta = with pkgs.lib; {
              description = "Android Client App for RSS (Miniflux) feeds";
              homepage = "https://github.com/wallago/rssh";
              license = [
                licenses.mit
                licenses.asl20
              ];
            };
          };
        claude = claude-code.packages.${system}.default;
      in
      rec {
        # ── Packages ──────────────────────────────────────────────
        packages = rec {
          rssh = buildApp { release = true; };
          rssh-debug = buildApp { release = false; };
          default = rssh;
        };

        # ── Checks (nix flake check) ─────────────────────────────
        checks.check = packages.rssh-debug;

        # ── Dev Shell (nix develop) ──────────────────────────────
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            rust-analyzer
            just
            wasm-bindgen-cli
            dioxus-cli
            claude
          ];
        };
      }
    );
}
