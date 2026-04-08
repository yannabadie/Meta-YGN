#!/bin/bash
set -e
echo "=== Installing Aletheia-Nexus ==="

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { echo "Error: Rust/Cargo required. Install from https://rustup.rs"; exit 1; }

# Build daemon and CLI
echo "Building daemon and CLI..."
cargo build --workspace --release

# Find the binary, checking multiple possible locations
find_binary() {
    local name=$1
    for dir in target/release target/debug target/*/release target/*/debug; do
        if [ -f "$dir/$name" ]; then
            echo "$dir/$name"
            return 0
        fi
    done
    return 1
}

# Create symlinks or copy to PATH
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

for bin in aletheiad aletheiad.exe aletheia aletheia.exe; do
    BIN_PATH=$(find_binary "$bin") && cp "$BIN_PATH" "$INSTALL_DIR/" 2>/dev/null || true
done

echo "Binaries installed to $INSTALL_DIR"
echo ""
echo "To use as Claude Code plugin:"
echo "  claude --plugin-dir $(pwd)"
echo ""
echo "To start the daemon:"
echo "  aletheia start"
echo ""
echo "=== Installation complete ==="
