#!/usr/bin/env bash
# Run the Degree example through a real pluma binary.
#
# Steps:
#   1. Build libdegree_rs.so in release mode.
#   2. Stage it under a temporary PLUMA_PLUGIN_PATH so pluma can discover
#      Degree without modifying the host PluMA tree.
#   3. Invoke pluma against example/config.txt from the repo root.
#   4. Diff the produced output against example/expected_output.txt.
#
# Usage:
#     ./run_pluma_example.sh                # uses `pluma` from $PATH
#     ./run_pluma_example.sh /path/to/pluma # explicit binary
#     PLUMA=/path/to/pluma ./run_pluma_example.sh
#
# Requirements:
#   * `pluma` binary on $PATH or via $1/$PLUMA. Must be built with
#     `--with-rust` and the HAVE_RUST fix from FIUBioRG/PluMA#13 (without
#     that fix the Rust loader is #ifdef-stripped and Degree silently won't
#     run).

set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
PLUMA_BIN="${1:-${PLUMA:-pluma}}"

if ! command -v "$PLUMA_BIN" >/dev/null 2>&1 && ! [ -x "$PLUMA_BIN" ]; then
    echo "error: pluma binary not found ('$PLUMA_BIN'). Pass a path or set \$PLUMA." >&2
    exit 1
fi

cd "$ROOT"
echo "→ cargo build --release"
cargo build --release --quiet

PLUGIN_DIR="$(mktemp -d)"
trap 'rm -rf "$PLUGIN_DIR"' EXIT
mkdir -p "$PLUGIN_DIR/Degree"
# PluMA's Rust loader globs <name>/Cargo.toml + <name>/lib<name>Plugin.so
ln -sfn "$ROOT/target/release/libdegree_rs.so" "$PLUGIN_DIR/Degree/libDegreePlugin.so"
ln -sfn "$ROOT/Cargo.toml"                     "$PLUGIN_DIR/Degree/Cargo.toml"

mkdir -p example/out
rm -f example/out/degree.txt

echo "→ pluma example/config.txt"
PLUMA_PLUGIN_PATH="$PLUGIN_DIR" "$PLUMA_BIN" example/config.txt

echo
echo "→ diff example/out/degree.txt example/expected_output.txt"
if diff -q example/out/degree.txt example/expected_output.txt >/dev/null; then
    echo "✓ output matches expected_output.txt"
else
    echo "✗ output differs from expected_output.txt"
    diff example/out/degree.txt example/expected_output.txt | head -20
    exit 1
fi
