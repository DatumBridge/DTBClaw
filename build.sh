#!/usr/bin/env bash
# Install the OctoClaw CLI from this repo into Cargo's bin directory (~/.cargo/bin by default).
# Uses the release profile (cargo install default). --force replaces an existing install.
#
# Usage:
#   ./build.sh
#   ./build.sh --locked
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

exec cargo install --path . --force "$@"
