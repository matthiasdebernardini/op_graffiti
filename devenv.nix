{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
	packages = [
	    pkgs.git
		pkgs.jq
		pkgs.just
		pkgs.openssl
		pkgs.cargo-nextest
		pkgs.cargo-watch
		pkgs.cargo-wizard
		pkgs.cargo-readme
		pkgs.hurl
		]
		++
		lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
				frameworks.CoreFoundation
				frameworks.Security
				frameworks.SystemConfiguration
		]);

	enterShell = ''
		cargo --version
		alias c="cargo"
		alias cc="cargo check"
		alias crk="cargo run --bin keel"
		alias cck="cargo check --bin keel"
		alias crs="cargo run --bin siren"
		'';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep "2.42.0"
  '';

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/languages/
  # languages.nix.enable = true;

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # https://devenv.sh/processes/
  # processes.ping.exec = "ping example.com";

  # See full reference at https://devenv.sh/reference/options/
}
