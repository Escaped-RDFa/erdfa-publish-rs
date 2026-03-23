{
  description = "DA51 Lean4 library — CborVal, Encode, Reflect, Conformal, Ternary";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [ pkgs.elan ];
        shellHook = ''
          export LEAN_PATH="$(pwd)"
          echo "DA51 lean4-lib — run 'make check' to type-check, 'make test' to run tests"
        '';
      };
    };
}
