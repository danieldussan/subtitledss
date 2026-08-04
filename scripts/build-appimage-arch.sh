#!/usr/bin/env bash
# Build the Linux AppImage on Arch Linux.
#
# Tauri bundles an old linuxdeploy whose `strip` cannot handle RELR sections
# (binutils >= 2.41, "unknown type [0x13] section .relr.dyn"), and whose gtk
# plugin assumes a Debian-style gdk-pixbuf loader directory that does not exist
# on Arch (loaders are compiled into libgdk_pixbuf). This script:
#
#   1. sets NO_STRIP=1 so linuxdeploy skips stripping;
#   2. patches the cached gtk plugin script (~/.cache/tauri) to tolerate a
#      missing gdk-pixbuf dir;
#   3. runs `tauri build --bundles appimage`.
#
# The patch touches only Tauri's tools cache, never the repository. On
# Debian/Ubuntu (e.g. CI) none of this is needed.
set -euo pipefail

PLUGIN="${HOME}/.cache/tauri/linuxdeploy-plugin-gtk.sh"
cd "$(dirname "$0")/.."

patch_plugin() {
  if [ ! -f "$PLUGIN" ]; then
    echo "gtk plugin not found at $PLUGIN; run 'bunx tauri build --bundles appimage' once first" >&2
    exit 1
  fi
  if grep -q "pixbuf_ok" "$PLUGIN"; then
    echo "gtk plugin already patched"
    return
  fi
  python3 - "$PLUGIN" <<'PY'
import sys

p = sys.argv[1]
s = open(p).read()

old1 = 'copy_tree "$gdk_pixbuf_binarydir" "$APPDIR/"'
new1 = '''if [ -d "$gdk_pixbuf_binarydir" ]; then
    copy_tree "$gdk_pixbuf_binarydir" "$APPDIR/"
    pixbuf_ok=1
fi'''
assert old1 in s
s = s.replace(old1, new1, 1)

old2 = 'cat >> "$HOOKFILE" <<EOF\nexport GDK_PIXBUF_MODULE_FILE="\\$APPDIR/$gdk_pixbuf_cache_file"\nEOF'
new2 = 'if [ "$pixbuf_ok" = 1 ]; then\ncat >> "$HOOKFILE" <<EOF\nexport GDK_PIXBUF_MODULE_FILE="\\$APPDIR/$gdk_pixbuf_cache_file"\nEOF\nfi'
assert old2 in s
s = s.replace(old2, new2, 1)

old3 = '"$gdk_pixbuf_query" > "$APPDIR/$gdk_pixbuf_cache_file"'
new3 = '[ "$pixbuf_ok" = 1 ] && "$gdk_pixbuf_query" > "$APPDIR/$gdk_pixbuf_cache_file"'
assert old3 in s
s = s.replace(old3, new3, 1)

old4 = 'sed -i "s|$gdk_pixbuf_moduledir/||g" "$APPDIR/$gdk_pixbuf_cache_file"'
new4 = '[ -f "$APPDIR/$gdk_pixbuf_cache_file" ] && sed -i "s|$gdk_pixbuf_moduledir/||g" "$APPDIR/$gdk_pixbuf_cache_file"'
assert old4 in s
s = s.replace(old4, new4, 1)

open(p, "w").write(s)
PY
  echo "gtk plugin patched"
}

patch_plugin
NO_STRIP=1 bunx tauri build --bundles appimage
