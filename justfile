# Meta-YGN cross-language task runner
# Install just: cargo install just

# === Rust ===

# Check all Rust crates
check:
    cargo check --workspace

# Test all Rust crates
test-rust:
    cargo test --workspace

# Build release binaries
build-rust:
    cargo build --workspace --release

# Run the daemon (dev mode)
daemon:
    cargo run -p metaygn-daemon

# Run the CLI
cli *ARGS:
    cargo run -p metaygn-cli -- {{ARGS}}

# === TypeScript ===

# Install TS dependencies
install-ts:
    pnpm install

# Type-check all TS packages
check-ts:
    cd packages/shared && npx tsc --noEmit
    cd packages/hooks && npx tsc --noEmit

# === Cross-language ===

# Check everything
check-all: check check-ts

# Test everything
test-all: test-rust

# Full release build
build: build-rust

# Format Rust code
fmt:
    cargo fmt --all

# Clean all build artifacts
clean:
    cargo clean
    rm -rf packages/*/dist packages/*/node_modules
