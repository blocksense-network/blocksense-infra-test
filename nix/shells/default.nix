{...}: {
  perSystem = {
    pkgs,
    inputs',
    ...
  }: {
    devShells.default = import ./nodejs.nix {inherit pkgs inputs';};
  };
}
