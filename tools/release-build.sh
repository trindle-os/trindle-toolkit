#!/bin/sh
# Builds the toolkit's programs for shipping with no path from the build computer in them; prints the build's own folder.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cargo_home=${CARGO_HOME:-$HOME/.cargo}
target=${CARGO_TARGET_DIR:-$root/target/shipped}
export CARGO_TARGET_DIR=$target
RUSTFLAGS="${RUSTFLAGS:-} --cfg trindle_shipped --remap-path-prefix=$HOME=/home --remap-path-prefix=$(dirname "$root")=/src --remap-path-prefix=$cargo_home=/cargo --remap-path-prefix=$target=/target"
export RUSTFLAGS
cd "$root"
cargo +1.85 build --release --locked --bins "$@"
printf '%s\n' "$target/release"
