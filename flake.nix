{
  description = "A straight-forward template renderer";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    inherit (nixpkgs) lib;
    withSystem = f:
      lib.fold lib.recursiveUpdate {}
      (map f ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"]);
  in
    withSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        inherit (pkgs) stdenv lib;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        buildDeps = with pkgs; (
          lib.optionals stdenv.isDarwin [
            libiconv
          ]
        );
        crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          nativeBuildInputs = buildDeps;
        };

        makeScriptWriter = pkgs.writers.makeScriptWriter;
        writeNuStdin = name: argsOrScript:
          if lib.isAttrs argsOrScript && !lib.isDerivation argsOrScript
          then
            makeScriptWriter (argsOrScript
              // {
                interpreter = "${lib.getExe pkgs.nushell} --no-config-file --stdin";
              })
            name
          else
            makeScriptWriter {
              interpreter = "${lib.getExe pkgs.nushell} --no-config-file --stdin";
            }
            name
            argsOrScript;
        writeNuStdinBin = name: writeNuStdin "/bin/${name}";

        tintedConverterDeps = with pkgs; [pastel];
        tintedConverter =
          writeNuStdinBin "convert-tinted-scheme"
          {
            # Add runtime deps to PATH for the script:
            makeWrapperArgs = [
              "--prefix"
              "PATH"
              ":"
              (pkgs.lib.makeBinPath tintedConverterDeps)
            ];
          }
          (builtins.readFile ./extra/convert-tinted-scheme);
      in {
        apps.${system}.default = let
          name = crate.pname or crate.name;
          exe = crate.passthru.exePath or "/bin/${name}";
        in {
          type = "app";
          program = "${crate}${exe}";
        };
        packages.${system} = {
          default = crate;
          convert-tinted-scheme = tintedConverter;
        };
        checks.${system} = {inherit crate;};
        devShells.${system}.default = pkgs.mkShell {
          packages = with pkgs;
            [
              toolchain
              rust-analyzer-unwrapped
            ]
            ++ buildDeps ++ tintedConverterDeps;
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
