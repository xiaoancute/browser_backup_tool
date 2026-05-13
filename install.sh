#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  cat <<'USAGE'
Usage:
  ./install.sh

Installs browser_backup_tool from this source checkout using Cargo.
The binary is installed to Cargo's bin directory, usually ~/.cargo/bin.
USAGE
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is not installed. Install Rust first: https://rustup.rs/" >&2
  exit 1
fi

cargo install --path "$script_dir" --force

cat <<'DONE'
Installed.

Run:
  browser_backup_tool

If the command is not found, add Cargo's bin directory to PATH:
  export PATH="$HOME/.cargo/bin:$PATH"
DONE
