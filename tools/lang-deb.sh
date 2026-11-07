#!/bin/sh
# Builds trindle-toolkit-lang_VERSION_all.deb: the toolkit's language files at /usr/share/trindle-toolkit/lang.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -n 1)
deb_version=$(printf '%s' "$version" | sed 's/-/~/')
out=${1:-$root/target/deb}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

lang=$work/usr/share/trindle-toolkit/lang
doc=$work/usr/share/doc/trindle-toolkit-lang
mkdir -p "$lang" "$doc" "$work/DEBIAN" "$out"
for dir in "$root"/share/lang/*/; do
    code=$(basename "$dir")
    mkdir -p "$lang/$code"
    cp "$dir"*.ftl "$lang/$code/"
done
cp "$root/debian/copyright" "$doc/copyright"
find "$work/usr" -type d -exec chmod 755 {} +
find "$work/usr" -type f -exec chmod 644 {} +

size=$(du -sk "$work/usr" | cut -f1)
cat > "$work/DEBIAN/control" <<EOF
Package: trindle-toolkit-lang
Version: $deb_version
Architecture: all
Maintainer: Trindle <toolkit@trindle.org>
Section: localization
Priority: optional
Installed-Size: $size
Description: Words of the Trindle Toolkit in each language
 The translated words the toolkit's own controls show, for programs built on
 the Trindle Toolkit. English is built into each program.
EOF

deb=$out/trindle-toolkit-lang_${deb_version}_all.deb
dpkg-deb --root-owner-group --build "$work" "$deb" >/dev/null
printf '%s\n' "$deb"
