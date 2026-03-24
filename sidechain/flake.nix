{
  description = "eRDFa Solana Sidechain — test-validator + stego-gossip P2P layer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = { url = "github:nix-community/fenix"; inputs.nixpkgs.follows = "nixpkgs"; };
    naersk = { url = "github:nix-community/naersk"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = { self, nixpkgs, fenix, naersk }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = fenix.packages.${system}.stable.toolchain;
      naersk' = naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };

      stego-gossip = naersk'.buildPackage {
        src = ./..;
        cargoBuildOptions = x: x ++ ["--bin" "stego-gossip"];
      };
    in {
      packages.${system} = {
        default = stego-gossip;
        stego-gossip = stego-gossip;
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          pkgs.solana-cli
          toolchain
          pkgs.minizinc
        ];
        shellHook = ''
          echo "◎ eRDFa Solana Sidechain"
          echo "========================"
          echo ""
          echo "  solana-test-validator    — start local validator"
          echo "  cargo run --bin stego-gossip -- --help"
          echo "  ./sidechain/launch.sh    — start everything"
          echo ""
          solana --version 2>/dev/null || true
        '';
      };

      # NixOS module for systemd services
      nixosModules.default = { config, lib, pkgs, ... }: {
        systemd.services.solana-sidechain = {
          description = "eRDFa Solana Sidechain Validator";
          after = [ "network.target" ];
          wantedBy = [ "multi-user.target" ];
          serviceConfig = {
            Type = "simple";
            ExecStart = "${pkgs.solana-cli}/bin/solana-test-validator --ledger /var/lib/erdfa-sidechain/ledger --rpc-port 8899 --gossip-port 8001 --faucet-port 9900 --log -";
            Restart = "on-failure";
            StateDirectory = "erdfa-sidechain";
          };
        };

        systemd.services.stego-gossip = {
          description = "eRDFa Stego Gossip P2P Layer";
          after = [ "solana-sidechain.service" ];
          wantedBy = [ "multi-user.target" ];
          serviceConfig = {
            Type = "simple";
            ExecStart = "${stego-gossip}/bin/stego-gossip --rpc http://127.0.0.1:8899 --listen 0.0.0.0:7700 --peers /var/lib/erdfa-sidechain/peers.json";
            Restart = "on-failure";
            StateDirectory = "erdfa-sidechain";
          };
        };
      };
    };
}
