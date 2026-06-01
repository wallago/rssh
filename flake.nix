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

        # ── Android SDK + NDK ─────────────────────────────────────
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          includeNDK = true;
          platformVersions = [ "34" ];
          ndkVersions = [ "29.0.14206865" ];
          buildToolsVersions = [ "34.0.0" ];
          includeEmulator = true;
          includeSystemImages = true;
          systemImageTypes = [ "google_apis" ];
          abiVersions = [ "x86_64" ];
        };
        androidSdk = androidComposition.androidsdk;
        androidSdkRoot = "${androidSdk}/libexec/android-sdk";
        ndkVersion = "29.0.14206865";
        jdk = pkgs.jdk25;

        # ── Claude Settings ─────────────────────────────────────
        claudeLocalSettings = builtins.toJSON {
          permissions = {
            allow = [
              # Nix
              "Bash(nix flake check*)"
              "Bash(nix eval*)"
              "Bash(nixos-rebuild dry-build*)"
              "Bash(statix check*)"
              "Bash(deadnix*)"
              "Bash(just*)"
              "Bash(nix build --dry-run*)"
              "Bash(nix search nixpkgs*)"
              "Bash(curl -s https://search.nixos.org*)"

              # Rust
              "Bash(cargo check*)"
              "Bash(cargo clippy*)"
              "Bash(cargo nextest run*)"
              "Bash(cargo test*)"
              "Bash(cargo tree*)"
              "Bash(cargo machete*)"
              "Bash(cargo deny check*)"
              "Bash(cargo audit*)"
            ];
          };
          enabledPlugins = {
            "claude-md-management@claude-plugins-official" = true;
            "superpowers@claude-plugins-official" = true;
            "context7@claude-plugins-official" = true;
            "code-review@claude-plugins-official" = true;
            "code-simplifier@claude-plugins-official" = true;
            "github@claude-plugins-official" = true;
            "frontend-design@claude-plugins-official" = true;
          };
        };
      in
      rec {
        # ── Packages ──────────────────────────────────────────────
        packages = rec {
          rssh = buildApp { release = true; };
          rssh-debug = buildApp { release = false; };
          default = rssh;
          android-emulator = pkgs.androidenv.emulateApp {
            name = "emulate";
            platformVersion = "34";
            abiVersion = "x86_64";
            systemImageType = "google_apis";
          };
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
            androidSdk
            jdk
            nodejs
          ];
          ANDROID_HOME = androidSdkRoot;
          ANDROID_SDK_ROOT = androidSdkRoot;
          ANDROID_NDK_HOME = "${androidSdkRoot}/ndk/${ndkVersion}";
          ANDROID_NDK_ROOT = "${androidSdkRoot}/ndk/${ndkVersion}";
          JAVA_HOME = "${jdk.home}";
          GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdkRoot}/build-tools/34.0.0/aapt2";
          shellHook = ''
            mkdir -p .claude
            echo '${claudeLocalSettings}' > .claude/settings.local.json
          '';
        };
      }
    );
}
