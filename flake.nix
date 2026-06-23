{
  description = "Blocksense Network Monorepo";

  nixConfig = {
    extra-substituters = [
      "https://cache.metacraft-labs.com/blocksense-public"
      "https://cache.metacraft-labs.com/metacraft-public"
    ];
    extra-trusted-public-keys = [
      "blocksense-public:OOgTc0ye1FONCiVHMrbpScc/HP+lX3uoU0EfwzX6ypE="
      "metacraft-public:UtS6PK+p0uZaJK3i/jD2DQOjTpddhQUQmNQDQih5N4Q="
    ];
  };

  inputs = {
    mcl-blockchain.url = "github:metacraft-labs/nix-blockchain-development";
    nixpkgs.follows = "mcl-blockchain/nixpkgs";
    nixpkgs-unstable.follows = "mcl-blockchain/nixpkgs-unstable";
    mcl-nixos-modules.follows = "mcl-blockchain/nixos-modules";
    ethereum-nix.follows = "mcl-blockchain/nixos-modules/ethereum-nix";
    flake-parts.follows = "mcl-blockchain/flake-parts";
    fenix.follows = "mcl-blockchain/fenix";
    devenv.follows = "mcl-blockchain/devenv";
    nix2container.follows = "mcl-blockchain/nix2container";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        # Third-party flake-parts modules
        inputs.devenv.flakeModule

        # Local flake-parts modules
        ./nix
      ];
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];
    };
}
