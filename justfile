# FaultLab — development commands

# Default: show available commands
default:
    @just --list

# Start all development services (web + gleam server)
dev:
    cd apps/web && npm run dev &
    cd services/collaboration-server && gleam run &
    wait

# Start only the web frontend
dev-web:
    cd apps/web && npm run dev

# Start only the Gleam collaboration server
dev-server:
    cd services/collaboration-server && gleam run

# Build the Rust workspace (native)
build-rust:
    cargo build

# Build the Rust workspace for WebAssembly
build-wasm:
    cargo build -p simulation-wasm --target wasm32-unknown-unknown

# Run Rust tests
test-rust:
    cargo test

# Run Gleam tests
test-gleam:
    cd services/collaboration-server && gleam test

# Run all tests
test: test-rust test-gleam

# Lint Rust code
lint-rust:
    cargo clippy -- -D warnings

# Format Rust code
fmt-rust:
    cargo fmt

# Format Gleam code
fmt-gleam:
    cd services/collaboration-server && gleam format

# Format everything
fmt: fmt-rust fmt-gleam

# Clean build artifacts
clean:
    cargo clean
    cd services/collaboration-server && gleam clean
    rm -rf apps/web/dist

# Install web dependencies
install-web:
    cd apps/web && npm install
