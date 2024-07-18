# Use a base image with Nix installed
FROM nixos/nix:latest as final

# Install devenv
RUN nix-env -iA devenv -f https://github.com/NixOS/nixpkgs/tarball/nixpkgs-unstable


# Install git (required for devenv)
RUN nix-env -iA nixpkgs.git

# Set up the working directory
WORKDIR /app


# Set up the devenv environment
RUN devenv

# Activate the devenv environment
RUN echo 'eval "$(devenv shell)"' >> ~/.bashrc

# Set the default command to run when the container starts
CMD ["cargo run"]