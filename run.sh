#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'USAGE'
Usage:
  ./run.sh             Run the TUI in debug mode
  ./run.sh --release   Run the TUI in release mode

Requires Rust and Cargo.
USAGE
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is not installed. Install Rust first: https://rustup.rs/" >&2
  exit 1
fi

if [[ "${1:-}" == "--release" ]]; then
  shift
  exec cargo run --release -- "$@"
fi

exec cargo run -- "$@"
