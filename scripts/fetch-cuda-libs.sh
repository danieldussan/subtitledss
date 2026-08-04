#!/usr/bin/env bash
set -euo pipefail

# Fetch sherpa-onnx CUDA provider libraries for bundling in releases.
# These files are NOT committed to git (too large for GitHub).
# Run this before `cargo tauri build` if you want GPU support in packages.

VENDOR_DIR="$(cd "$(dirname "$0")/../src-tauri/vendor/sherpa" && pwd)"
CUDA_LIB="$VENDOR_DIR/libonnxruntime_providers_cuda.so"
SHARED_LIB="$VENDOR_DIR/libonnxruntime_providers_shared.so"

SHERPA_VERSION="1.13.4"
TARBALL="sherpa-onnx-v${SHERPA_VERSION}-cuda-12.x-cudnn-9.x-linux-x64-gpu.tar.bz2"
URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/v${SHERPA_VERSION}/${TARBALL}"

if [ -f "$CUDA_LIB" ] && [ -f "$SHARED_LIB" ]; then
    echo "CUDA libs already present in $VENDOR_DIR"
    exit 0
fi

echo "Downloading sherpa-onnx CUDA libs v${SHERPA_VERSION}..."
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

wget -q --show-progress -O "$TMPDIR/$TARBALL" "$URL"
tar xjf "$TMPDIR/$TARBALL" -C "$TMPDIR"

cp "$TMPDIR/sherpa-onnx-v${SHERPA_VERSION}-cuda-12.x-cudnn-9.x-linux-x64-gpu/lib/libonnxruntime_providers_cuda.so" "$VENDOR_DIR/"
cp "$TMPDIR/sherpa-onnx-v${SHERPA_VERSION}-cuda-12.x-cudnn-9.x-linux-x64-gpu/lib/libonnxruntime_providers_shared.so" "$VENDOR_DIR/"

echo "CUDA libs installed to $VENDOR_DIR"
ls -lh "$VENDOR_DIR"/libonnxruntime_providers_*.so
