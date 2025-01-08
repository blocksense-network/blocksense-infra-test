{ inputs, self, ... }:
{
  perSystem =
    { pkgs, self', ... }:
    let
      rust = self'.legacyPackages.rustToolchain;

      craneLib = (inputs.mcl-blockchain.inputs.crane.mkLib pkgs).overrideToolchain rust;

      version = "dev";

      blocksense-rs = pkgs.callPackage ./blocksense-rs {
        inherit craneLib version;
        inherit (self.lib) filesets;
      };

      mkOracleScript =
        oracle-name:
        pkgs.callPackage ./oracle-script {
          inherit craneLib version oracle-name;
          inherit (self.lib) filesets;
        };

      mkApp = package: exeName: {
        type = "app";
        program = "${package}/bin/${exeName}";
      };
    in
    {
      apps = {
        sequencer = mkApp blocksense-rs "sequencer";
        reporter = mkApp blocksense-rs "launch_reporter";
        blocksense = mkApp blocksense-rs "blocksense";
      };
      packages = {
        inherit blocksense-rs;
      };
      legacyPackages = {
        oracle-scripts = {
          cmc-wasm = mkOracleScript "cmc";
          yahoo-wasm = mkOracleScript "yahoo";
        };
      };
    };
}
