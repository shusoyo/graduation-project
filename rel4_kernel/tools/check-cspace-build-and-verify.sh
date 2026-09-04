#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERIFY_TARGET="${VERIFY_TARGET:-riscv64gc-unknown-none-elf}"
VERIFY_PACKAGE="${VERIFY_PACKAGE:-sel4_cspace}"
PLATFORM="${PLATFORM:-spike}"
MARCOS="${MARCOS:-KERNEL_STACK_BITS=12 FASTPATH=true HAVE_FPU=true RISCV_EXT_D=true}"

echo "[check-cspace] package=$VERIFY_PACKAGE target=$VERIFY_TARGET platform=$PLATFORM"
echo "[check-cspace] step 1/2: cargo check"
(
    cd "$ROOT_DIR"
    env \
        PLATFORM="$PLATFORM" \
        MARCOS="$MARCOS" \
        cargo check -p "$VERIFY_PACKAGE" --target "$VERIFY_TARGET"
)

echo "[check-cspace] step 2/2: cargo xtask verify"
(
    cd "$ROOT_DIR"
    env \
        PLATFORM="$PLATFORM" \
        MARCOS="$MARCOS" \
        cargo xtask verify --package "$VERIFY_PACKAGE" --jobs 1 --max-errors 50
)

echo "[check-cspace] build and verification checks passed"
