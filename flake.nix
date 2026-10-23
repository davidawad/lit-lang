{
  description = "litgraph — litigation procedure graphs as stochastic games";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
    }:
    flake-utils.lib.eachSystem
      [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ]
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          # Match rust-toolchain.toml exactly so `nix build` and `cargo build`
          # use the same compiler.
          rustToolchain = pkgs.rust-bin.stable."1.97.1".default.override {
            extensions = [
              "rustfmt"
              "clippy"
              "llvm-tools-preview"
            ];
            targets = [ "wasm32-unknown-unknown" ];
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);

          src = craneLib.cleanCargoSource ./.;

          # Build from the whole repo minus build outputs and bulky media, not an
          # allow-list: build.rs embeds data directories and tests read
          # examples/ and tests/, and an allow-list silently drifted twice.
          excluded = [ "target" ".worktrees" "result" "media" "marketing" ];
          unfilteredSrc = pkgs.lib.cleanSourceWith {
            src = pkgs.lib.cleanSource ./.;
            filter = path: _type: !(builtins.elem (baseNameOf path) excluded);
          };

          # litigate-smt's build.rs links the pinned Z3 release's static
          # libz3.a. The Nix sandbox has no network, so fetch the same zip
          # here and hand it over offline; build.rs re-checks the SHA-256.
          # Keep in step with ASSETS in crates/litigate-smt/build.rs.
          z3Version = "5.1.0";
          z3Asset =
            {
              x86_64-linux = {
                name = "z3-5.1.0-x64-glibc-2.39.zip";
                sha256 = "f47be8d27d3230e823bf1eeede2fe0abaca55bb78d0b59974370e6689a92284a";
              };
              aarch64-linux = {
                name = "z3-5.1.0-arm64-glibc-2.38.zip";
                sha256 = "2832cfd43c6862fbaf78834b5f1e35d031b4a87e85456b696c1c6f609cd5609e";
              };
              x86_64-darwin = {
                name = "z3-5.1.0-x64-osx-13.3.zip";
                sha256 = "f25cabdb01a32942a42227c36902c0846c759dcaabef4aca3b2e86331447f6ff";
              };
              aarch64-darwin = {
                name = "z3-5.1.0-arm64-osx-13.3.zip";
                sha256 = "81d29e934fd863079a74af35eecaeaef8047e0e12414d33ca322b358d68383db";
              };
            }
            .${system};
          z3Archive = pkgs.fetchurl {
            url = "https://github.com/Z3Prover/z3/releases/download/z3-${z3Version}/${z3Asset.name}";
            inherit (z3Asset) sha256;
          };

          commonArgs = {
            src = unfilteredSrc;
            strictDeps = true;
            LITIGATE_SMT_Z3_ARCHIVE = "${z3Archive}";
            nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          litgraph = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "litgraph";
              cargoExtraArgs = "-p litgraph-cli";
              doCheck = false; # `nix flake check` runs the real test suite separately.

              postInstall = ''
                mkdir -p $out/share/litgraph
                # The packs are build outputs of crates/litgraph-packs.
                built="$(find target -path '*/litgraph-packs-*/out/packs' -type d | head -n 1)"
                test -n "$built"
                cp -r "$built" $out/share/litgraph/packs
              '';

              meta = with pkgs.lib; {
                description = "Litigation procedure graphs as stochastic games: load, compose, weight, solve, simulate.";
                homepage = "https://github.com/davidawad/litgraph";
                license = licenses.gpl3Plus;
                mainProgram = "litgraph";
              };
            }
          );

          litgraphMcp = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "litgraph-mcp";
              cargoExtraArgs = "-p litgraph-mcp";
              doCheck = false; # `nix flake check` runs the real test suite separately.

              meta = with pkgs.lib; {
                description = "MCP (Model Context Protocol) stdio server for litgraph: every engine op as a tool, packs/links.json/the manual as resources.";
                homepage = "https://github.com/davidawad/litgraph";
                license = licenses.gpl3Plus;
                mainProgram = "litgraph-mcp";
              };
            }
          );

          # --- wasm: litgraph-wasm for web + nodejs, via wasm-bindgen -----------
          wasmArgs = commonArgs // {
            pname = "litgraph-wasm";
            cargoExtraArgs = "-p litgraph-wasm --target wasm32-unknown-unknown";
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            doCheck = false; # host can't execute wasm32 test binaries without wasmtime
          };
          wasmCargoArtifacts = craneLib.buildDepsOnly wasmArgs;
          litgraphWasmRaw = craneLib.buildPackage (
            wasmArgs // { cargoArtifacts = wasmCargoArtifacts; }
          );

          # wasm-bindgen-cli must match the `wasm-bindgen` crate version pinned in
          # crates/litgraph-wasm/Cargo.toml (=0.2.127) exactly -- a mismatch fails
          # at wasm-bindgen invocation time, not at cargo build time.
          litgraphWasm = pkgs.stdenv.mkDerivation {
            pname = "litgraph-wasm";
            version = "0.1.0";
            src = litgraphWasmRaw;
            nativeBuildInputs = [ pkgs.wasm-bindgen-cli ];
            buildPhase = ''
              runHook preBuild
              mkdir -p $out/web $out/node
              wasm_file=$(find $src -name "litgraph_wasm.wasm" | head -1)
              wasm-bindgen --target web --out-dir $out/web "$wasm_file"
              wasm-bindgen --target nodejs --out-dir $out/node "$wasm_file"
              runHook postBuild
            '';
            dontInstall = true;
            dontFixup = true;
          };

          devTools = with pkgs; [
            rustToolchain
            cargo-nextest
            cargo-llvm-cov
            cargo-audit
            cargo-deny
            cargo-machete
            just
            jq
            wasm-bindgen-cli
            wasm-pack
            # kani is not packaged in nixpkgs; install via `cargo install
            # kani-verifier && cargo kani setup` outside the sandbox, or run
            # `just verify-kani` from a non-nix shell / CI's cargo-kani action.
            # `cargo install wasm-pack` fails to link locally (-lbz2 missing)
            # on at least one dev machine -- use this nix-provided binary.
          ];
        in
        {
          packages.default = litgraph;
          packages.litgraph = litgraph;
          packages.litgraph-mcp = litgraphMcp;
          packages.wasm = litgraphWasm;

          apps.default = flake-utils.lib.mkApp { drv = litgraph; };
          apps.litgraph-mcp = flake-utils.lib.mkApp { drv = litgraphMcp; };

          devShells.default = pkgs.mkShell {
            packages = devTools;
            # No LITGRAPH_PACKS: the packs are build outputs embedded in the
            # binary (crates/litgraph-packs); `just lit-sync` writes a copy
            # to target/lit-build/packs for `--packs-dir`.
          };

          checks = {
            build = litgraph;
            build-mcp = litgraphMcp;

            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--workspace --all-targets -- -D warnings";
              }
            );

            fmt = craneLib.cargoFmt { inherit src; };

            nextest = craneLib.cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
              }
            );
          };

          formatter = pkgs.nixfmt-rfc-style;
        }
      );
}
