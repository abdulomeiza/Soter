# On-Chain Module (Soroban Contracts)

This module contains Soroban smart contracts for Soter's on-chain escrow and claimable packages functionality.

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Soroban CLI
cargo install --locked soroban-cli
```

## 🛠 Build

To build the contracts for release (optimized for WASM):

```bash
cargo build --target wasm32-unknown-unknown --release
```

## 🧪 Test

Run the unit and integration tests:

```bash
cargo test
```

## 🧹 Quality Checks

Ensure your code is formatted and error-free:

```bash
# Format check
cargo fmt --check

# Linter check (fails on warnings)
cargo clippy -- -D warnings
```

## 🚀 Deploy

To deploy to a network (e.g., Testnet):

```bash
# 1. Configure identity (if not already done)
soroban config identity generate alice

# 2. Deploy contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/aid_escrow.wasm \
  --source alice \
  --network testnet
```