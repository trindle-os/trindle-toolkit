#!/bin/sh
# Builds and tests the toolkit alone. Uses the build libraries unpacked in
set -e
here=$(cd "$(dirname "$0")/.." && pwd)
env_file="${TRINDLE_DEVLIBS_ENV:-$here/../trindle/target/devlibs/env.sh}"
[ -f "$env_file" ] && eval "$(grep -E '^export (PKG_CONFIG_PATH|PKG_CONFIG_SYSROOT_DIR|RUSTFLAGS|CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS)=[^$`;&|]*$' "$env_file")"
mkdir -p "$here/target/tmp-build"
TMPDIR="$here/target/tmp-build" cargo +1.85 test --release --workspace --manifest-path "$here/Cargo.toml" "$@"
