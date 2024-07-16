{ pkgs, lib, config, inputs, ... }:

let
  customBitcoind = pkgs.callPackage ./bitcoind.nix {};
in
{
  env.DATABASE_URL="postgresql://sirendb_owner:2vDJjo9pGKiP@ep-sweet-shape-a5ouj5s7.us-east-2.aws.neon.tech/sirendb?sslmode=require";

 packages = [
    pkgs.git
    pkgs.just
    pkgs.openssl
    pkgs.cargo-nextest
    pkgs.hurl
    pkgs.sqlx-cli
    pkgs.tailwindcss
    pkgs.cargo-watch
    pkgs.electrs
    customBitcoind
  ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
    frameworks.CoreFoundation
    frameworks.Security
    frameworks.SystemConfiguration
  ]);

  enterShell = ''
    git --version
    cargo --version
    alias c="cargo"
    bitcoind --version  # Check Bitcoin Core version
    export BITCOIND_EXE=$(which bitcoind)
    echo $BITCOIND_EXE
    export ELECTRS_EXEC=$(which electrs)
    echo $ELECTRS_EXEC
  '';

  # https://devenv.sh/pre-commit-hooks/
  pre-commit.hooks = {
    clippy.enable = true;
    cargo-check.enable = true;
    rustfmt.enable = true;
  };

  # https://devenv.sh/languages/
  languages.nix.enable = true;
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  services.postgres = {
    enable = true;
    # initialDatabases = [{ name = "keel"; }];
  };
}