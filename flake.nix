{
  description = "eRDFa Publish — Semantic UI components as DA51 CBOR shards";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-ipfs = {
      url = "github:meta-introspector/rust-ipfs";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-ipfs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          erdfa-publish = pkgs.rustPlatform.buildRustPackage {
            pname = "erdfa-publish";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
            buildFeatures = [ "native" "cli" "ipfs" ];
            buildNoDefaultFeatures = true;

            unpackPhase = ''
              runHook preUnpack
              cp -r --no-preserve=mode $src erdfa-publish
              mkdir -p erdfa-publish/vendor
              cp -r --no-preserve=mode ${rust-ipfs} erdfa-publish/vendor/rust-ipfs
              sourceRoot=$PWD/erdfa-publish
              runHook postUnpack
            '';
          };

          default = self.packages.${system}.erdfa-publish;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ cargo rustc rust-analyzer rustfmt clippy ];
        };
      }
    );
}
