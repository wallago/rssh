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
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };

        # ── Toolchain ─────────────────────────────────────────────
        rust = pkgs.rust-bin.nightly.latest.default.override {
          targets = [
            "x86_64-linux-android"
            "aarch64-linux-android"
          ];
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

        # ── Android helper ──────────────────────────────────────────
        # androidComposition = pkgs.androidenv.composeAndroidPackages {
        #   abiVersions = [ "x86_64" ];
        #   includeNDK = true;
        #   ndkVersions = [ "25.2.9519653" ];
        #   platformVersions = [ "34" ];
        #   platformToolsVersion = "34.0.0";
        #   toolsVersion = "26.1.1";
        #   buildToolsVersions = [ "34.0.0" ];
        #   includeEmulator = true;
        #   includeSystemImages = true;
        #   systemImageTypes = [ "google_apis" ];
        # };
        # androidSdk = androidComposition.androidsdk;
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
            # androidSdk
            androidenv.androidPkgs.androidsdk
          ];
          # shellHook = ''
          #   export JAVA_HOME=${pkgs.jdk17}
          #   export QT_QPA_PLATFORM="xcb"
          #   export QT_PLUGIN_PATH="${pkgs.qt5.qtbase}/lib/qt-5.15/plugins"
          #   export LIBGL_ALWAYS_SOFTWARE=1
          #   export ANDROID_SDK_ROOT="${androidSdk}/libexec/android-sdk"
          #   export ANDROID_NDK_HOME="${androidSdk}/libexec/android-sdk/ndk-bundle"
          #   export NDK_HOME="$ANDROID_NDK_HOME"
          #   export NDK_HOME_TOOLCHAIN_BIN="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
          # '';
        };
      }
    );
}
