#!/usr/bin/env bash
# TDD coverage gate for library crates (exclude presentation client & thin server binary).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "installing cargo-llvm-cov..."
  cargo install cargo-llvm-cov --locked
fi

# Fail under 93% line coverage on core crates.
cargo llvm-cov --workspace \
  --exclude nnk_client \
  --exclude nnk_server \
  --fail-under-lines 93 \
  --summary-only
