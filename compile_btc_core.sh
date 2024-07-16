#!/bin/bash

# Exit on error
set -e

# Bitcoin Core version to compile
VERSION="v0.21.0"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required commands
for cmd in git make; do
    if ! command_exists $cmd; then
        echo "Error: $cmd is not installed. Please install it and try again."
        exit 1
    fi
done

# Create a directory for Bitcoin Core
mkdir -p ~/bitcoin-core
cd ~/bitcoin-core

if [ -d "bitcoin" ]; then
    echo "Bitcoin Core repository already exists. Updating..."
    cd bitcoin
    git fetch --all
else
    echo "Cloning Bitcoin Core repository..."
    if ! git clone https://github.com/bitcoin/bitcoin.git; then
        echo "Error: Failed to clone the repository. Please check your internet connection and try again."
        exit 1
    fi
    cd bitcoin
fi

echo "Fetching all tags..."
if ! git fetch --all --tags; then
    echo "Error: Failed to fetch tags. Please check your internet connection and try again."
    exit 1
fi

echo "Checking out version ${VERSION}..."
if git tag | grep -q "^${VERSION}$"; then
    if ! git checkout "${VERSION}"; then
        echo "Error: Failed to checkout ${VERSION}. Please check the error message above."
        exit 1
    fi
else
    echo "Error: Tag ${VERSION} does not exist. Available tags are:"
    git tag
    exit 1
fi

echo "Installing dependencies..."
if ! brew install automake berkeley-db@4 boost libevent libtool miniupnpc openssl@1.1 pkg-config python qt@5 zeromq; then
    echo "Error: Failed to install dependencies. Please check Homebrew and try again."
    exit 1
fi

echo "Preparing the build environment..."
export LDFLAGS="-L/usr/local/opt/openssl@1.1/lib"
export CPPFLAGS="-I/usr/local/opt/openssl@1.1/include"

echo "Running autogen script..."
if ! ./autogen.sh; then
    echo "Error: autogen.sh failed. Please check the output above for more details."
    exit 1
fi

echo "Configuring..."
if ! ./configure --with-gui=qt5; then
    echo "Error: Configuration failed. Please check the output above for more details."
    exit 1
fi

echo "Compiling (this may take a while)..."
if ! make -j4; then
    echo "Error: Compilation failed. Please check the output above for more details."
    exit 1
fi

echo "Installation (requires sudo access)..."
if ! sudo make install; then
    echo "Error: Installation failed. Please check if you have sudo privileges."
    exit 1
fi

echo "Bitcoin Core ${VERSION} has been compiled and installed."
echo "You can run 'bitcoind' to start the Bitcoin daemon or 'bitcoin-qt' to start the GUI."