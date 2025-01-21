#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

echo "Installing zkRust from source..."

BASE_DIR=$HOME
ZKRUST_DIR="${ZKRUST_DIR-"$BASE_DIR/.zkRust"}"
ZKRUST_BIN_DIR="$ZKRUST_DIR/bin"
ZKRUST_BIN_PATH="$ZKRUST_BIN_DIR/zkRust"

# Create bin directory
mkdir -p "$ZKRUST_BIN_DIR"

# Build from source using local code
echo "Building zkRust from source..."
cargo build --release
cp target/release/zkRust "$ZKRUST_BIN_PATH"

chmod +x "$ZKRUST_BIN_PATH"

# Store the correct profile file
case $SHELL in
*/zsh)
    PROFILE="${ZDOTDIR-"$HOME"}/.zshenv"
    PREF_SHELL=zsh
    ;;
*/bash)
    PROFILE=$HOME/.bashrc
    PREF_SHELL=bash
    ;;
*/fish)
    PROFILE=$HOME/.config/fish/config.fish
    PREF_SHELL=fish
    ;;
*/ash)
    PROFILE=$HOME/.profile
    PREF_SHELL=ash
    ;;
*)
    echo "zkrust: could not detect shell, manually add ${ZKRUST_BIN_DIR} to your PATH."
    exit 1
esac

# Only add to PATH if it isn't already there
if [[ ":$PATH:" != *":${ZKRUST_BIN_DIR}:"* ]]; then
    if [[ "$PREF_SHELL" == "fish" ]]; then
        echo >> "$PROFILE" && echo "fish_add_path -a $ZKRUST_BIN_DIR" >> "$PROFILE"
    else
        echo >> "$PROFILE" && echo "export PATH=\"\$PATH:$ZKRUST_BIN_DIR\"" >> "$PROFILE"
    fi
fi

echo "zkRust built and installed successfully in $ZKRUST_BIN_PATH"
echo "Detected your preferred shell is $PREF_SHELL and added to PATH."
echo "Installing zkVM toolchains"

# Check for RISC0 toolchain
echo "Checking for RISC0 toolchain..."
if ! command -v rzup &> /dev/null; then
    echo "Installing RISC0 toolchain..."
    curl -L https://risczero.com/install | bash
    export PATH="$PATH:$HOME/.risc0/bin"
    rzup install
else
    echo "RISC0 toolchain already installed"
fi
cargo risczero --version

# Check for SP1 toolchain
echo "Checking for SP1 toolchain..."
if ! command -v sp1up &> /dev/null; then
    echo "Installing SP1 toolchain..."
    curl -L https://sp1.succinct.xyz | bash
    export PATH="$PATH:$HOME/.sp1/bin"
    sp1up -v v4.0.0
else
    echo "SP1 toolchain already installed"
fi
cargo prove --version

# Set up workspaces directory
echo "Setting up workspaces..."
mkdir -p "$ZKRUST_DIR/workspaces"
cp -r "$SCRIPT_DIR/workspaces/"* "$ZKRUST_DIR/workspaces/"

echo "Run 'source $PROFILE' or start a new terminal session to use zkRust!" 