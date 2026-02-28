#!/bin/bash
set -e
echo "=== Installing Aletheia-Nexus ==="

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { echo "Error: Rust/Cargo required. Install from https://rustup.rs"; exit 1; }

# Build daemon and CLI
echo "Building daemon and CLI..."
cargo build --workspace --release

# Create symlinks or copy to PATH
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/aletheiad "$INSTALL_DIR/" 2>/dev/null || cp target/release/aletheiad.exe "$INSTALL_DIR/" 2>/dev/null
cp target/release/aletheia "$INSTALL_DIR/" 2>/dev/null || cp target/release/aletheia.exe "$INSTALL_DIR/" 2>/dev/null

echo "Binaries installed to $INSTALL_DIR"
echo ""
echo "To use as Claude Code plugin:"
echo "  claude --plugin-dir $(pwd)"
echo ""
echo "To start the daemon:"
echo "  aletheia start"
echo ""
echo "=== Installation complete ==="
